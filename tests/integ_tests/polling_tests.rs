use assert_matches::assert_matches;
use futures::future::join_all;
use std::time::Duration;
use temporal_sdk_core::prototype_rust_sdk::{WfContext, WorkflowResult};
use temporal_sdk_core_protos::coresdk::{
    activity_task::activity_task as act_task,
    workflow_activation::{wf_activation_job, FireTimer, WfActivationJob},
    workflow_commands::{ActivityCancellationType, RequestCancelActivity, StartTimer},
    workflow_completion::WfActivationCompletion,
    IntoCompletion,
};
use test_utils::{init_core_and_create_wf, schedule_activity_cmd, CoreTestHelpers, CoreWfStarter};
use tokio::time::timeout;

#[tokio::test]
async fn out_of_order_completion_doesnt_hang() {
    let (core, task_q) = init_core_and_create_wf("out_of_order_completion_doesnt_hang").await;
    let activity_id = "act-1";
    let task = core.poll_workflow_activation(&task_q).await.unwrap();
    // Complete workflow task and schedule activity and a timer that fires immediately
    core.complete_workflow_activation(
        vec![
            schedule_activity_cmd(
                0,
                &task_q,
                activity_id,
                ActivityCancellationType::TryCancel,
                Duration::from_secs(60),
                Duration::from_secs(60),
            ),
            StartTimer {
                seq: 1,
                start_to_fire_timeout: Some(Duration::from_millis(50).into()),
            }
            .into(),
        ]
        .into_completion(task_q.clone(), task.run_id),
    )
    .await
    .unwrap();
    // Poll activity and verify that it's been scheduled with correct parameters, we don't expect to
    // complete it in this test as activity is try-cancelled.
    let activity_task = core.poll_activity_task(&task_q).await.unwrap();
    assert_matches!(
        activity_task.variant,
        Some(act_task::Variant::Start(start_activity)) => {
            assert_eq!(start_activity.activity_type, "test_activity".to_string())
        }
    );
    // Poll workflow task and verify that activity has failed.
    let task = core.poll_workflow_activation(&task_q).await.unwrap();
    assert_matches!(
        task.jobs.as_slice(),
        [
            WfActivationJob {
                variant: Some(wf_activation_job::Variant::FireTimer(
                    FireTimer { seq: t_seq }
                )),
            },
        ] => {
            assert_eq!(*t_seq, 1);
        }
    );

    // Start polling again *before* we complete the WFT
    let cc = core.clone();
    let tq = task_q.clone();
    let jh = tokio::spawn(async move {
        // We want to fail the test if this takes too long -- we should not hit long poll timeout
        let task = timeout(Duration::from_secs(1), cc.poll_workflow_activation(&tq))
            .await
            .expect("Poll should come back right away")
            .unwrap();
        assert_matches!(
            task.jobs.as_slice(),
            [WfActivationJob {
                variant: Some(wf_activation_job::Variant::ResolveActivity(_)),
            }]
        );
        cc.complete_execution(&tq, &task.run_id).await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    // Then complete the (last) WFT with a request to cancel the AT, which should produce a
    // pending activation, unblocking the (already started) poll
    core.complete_workflow_activation(WfActivationCompletion::from_cmds(
        &task_q,
        task.run_id,
        vec![RequestCancelActivity { seq: 0 }.into()],
    ))
    .await
    .unwrap();

    jh.await.unwrap();
}

pub async fn many_parallel_timers_longhist(mut ctx: WfContext) -> WorkflowResult<()> {
    for _ in 0..20 {
        let mut futs = vec![];
        for _ in 0..1000 {
            futs.push(ctx.timer(Duration::from_millis(100)));
        }
        join_all(futs).await;
    }
    Ok(().into())
}

// Ignored for now because I can't actually get this to produce pages. Need to generate some
// large payloads I think.
#[tokio::test]
#[ignore]
async fn can_paginate_long_history() {
    let wf_name = "can_paginate_long_history";
    let mut starter = CoreWfStarter::new(wf_name);
    // Do not use sticky queues so we are forced to paginate once history gets long
    starter.max_cached_workflows(0);

    let mut worker = starter.worker().await;
    worker.register_wf(wf_name.to_owned(), many_parallel_timers_longhist);
    worker
        .submit_wf(wf_name.to_owned(), wf_name.to_owned(), vec![])
        .await
        .unwrap();
    worker.run_until_done().await.unwrap();
    starter.shutdown().await;
}
