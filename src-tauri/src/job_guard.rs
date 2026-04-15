// Windows JobObject guard: ensures any child process assigned to this job
// is killed when the parent muku.exe exits — including abrupt termination
// via Task Manager, crash, or user logoff.

#[cfg(windows)]
use std::process::Child;
#[cfg(windows)]
use std::sync::OnceLock;

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_BREAKAWAY_OK, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

#[cfg(windows)]
struct JobHandle(HANDLE);

#[cfg(windows)]
unsafe impl Send for JobHandle {}
#[cfg(windows)]
unsafe impl Sync for JobHandle {}

#[cfg(windows)]
static JOB: OnceLock<Option<JobHandle>> = OnceLock::new();

#[cfg(windows)]
fn init_job() -> Option<JobHandle> {
    unsafe {
        let job = CreateJobObjectW(None, windows::core::PCWSTR::null()).ok()?;
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_BREAKAWAY_OK;
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
        .ok()?;
        Some(JobHandle(job))
    }
}

#[cfg(windows)]
pub fn assign(child: &Child) -> std::io::Result<()> {
    let job_opt = JOB.get_or_init(init_job);
    let Some(job) = job_opt else {
        return Err(std::io::Error::other("JobObject not initialized"));
    };
    let handle = HANDLE(child.as_raw_handle());
    unsafe {
        AssignProcessToJobObject(job.0, handle)
            .map_err(|e| std::io::Error::other(format!("AssignProcessToJobObject failed: {e}")))?;
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn assign(_child: &std::process::Child) -> std::io::Result<()> {
    Ok(())
}
