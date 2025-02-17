use crate::integ_tests::workflow_tests::timers::timer_wf;
use std::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};
use temporal_sdk_core::prototype_rust_sdk::{WfContext, WorkflowResult};
use test_utils::CoreWfStarter;

#[tokio::test]
async fn timer_workflow_not_sticky() {
    let wf_name = "timer_wf_not_sticky";
    let mut starter = CoreWfStarter::new(wf_name);
    starter.max_cached_workflows(0);
    let mut worker = starter.worker().await;
    worker.register_wf(wf_name.to_owned(), timer_wf);

    worker
        .submit_wf(wf_name.to_owned(), wf_name.to_owned(), vec![])
        .await
        .unwrap();
    worker.run_until_done().await.unwrap();
    starter.shutdown().await;
}

static TIMED_OUT_ONCE: AtomicBool = AtomicBool::new(false);
static RUN_CT: AtomicUsize = AtomicUsize::new(0);
async fn timer_timeout_wf(mut ctx: WfContext) -> WorkflowResult<()> {
    RUN_CT.fetch_add(1, Ordering::SeqCst);
    let t = ctx.timer(Duration::from_secs(1));
    if !TIMED_OUT_ONCE.load(Ordering::SeqCst) {
        ctx.force_task_fail(anyhow::anyhow!("I AM SLAIN!"));
        TIMED_OUT_ONCE.store(true, Ordering::SeqCst);
    }
    t.await;
    Ok(().into())
}

#[tokio::test]
async fn timer_workflow_timeout_on_sticky() {
    // This test intentionally times out a workflow task in order to make the next task be scheduled
    // on a not-sticky queue
    let wf_name = "timer_workflow_timeout_on_sticky";
    let mut starter = CoreWfStarter::new(wf_name);
    starter.wft_timeout(Duration::from_secs(2));
    let mut worker = starter.worker().await;
    worker.register_wf(wf_name.to_owned(), timer_timeout_wf);

    worker
        .submit_wf(wf_name.to_owned(), wf_name.to_owned(), vec![])
        .await
        .unwrap();
    worker.run_until_done().await.unwrap();
    starter.shutdown().await;
    // If it didn't run twice it didn't time out
    assert_eq!(RUN_CT.load(Ordering::SeqCst), 2);
}
