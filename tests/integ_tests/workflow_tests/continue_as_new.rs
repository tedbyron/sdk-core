use std::time::Duration;
use temporal_sdk_core::prototype_rust_sdk::{WfContext, WfExitValue, WorkflowResult};
use temporal_sdk_core_protos::coresdk::workflow_commands::ContinueAsNewWorkflowExecution;
use test_utils::CoreWfStarter;

async fn continue_as_new_wf(mut ctx: WfContext) -> WorkflowResult<()> {
    let run_ct = ctx.get_args()[0].data[0];
    ctx.timer(Duration::from_millis(500)).await;
    Ok(if run_ct < 5 {
        WfExitValue::ContinueAsNew(ContinueAsNewWorkflowExecution {
            arguments: vec![[run_ct + 1].into()],
            ..Default::default()
        })
    } else {
        ().into()
    })
}

#[tokio::test]
async fn continue_as_new_happy_path() {
    let wf_name = "continue_as_new_happy_path";
    let mut starter = CoreWfStarter::new(wf_name);
    let mut worker = starter.worker().await;
    worker.register_wf(wf_name.to_string(), continue_as_new_wf);

    worker
        .submit_wf(wf_name.to_string(), wf_name.to_string(), vec![[1].into()])
        .await
        .unwrap();
    worker.run_until_done().await.unwrap();

    // Terminate the continued workflow
    starter
        .get_core()
        .await
        .server_gateway()
        .terminate_workflow_execution(wf_name.to_owned(), None)
        .await
        .unwrap();

    starter.shutdown().await;
}
