use crate::handles::CoreServiceHandles;
use crate::handlers::MonitoringHeartbeatHandler;

pub fn register(
    handles: &CoreServiceHandles,
) {

    handles.def.subscribe(
        "monitoring.heartbeat",
        MonitoringHeartbeatHandler {
            dai: handles.dai.clone(),
            die: handles.die.clone(),
            dde: handles.dde.clone(),
        },
    );
}