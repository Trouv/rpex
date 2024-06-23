use std::process::Command;

use clap::Parser;
use rpex::HyperRectangle;
use rpex::Partition;
use rpex::Rpex;
use rpex::SumsInRatioEvaluationError;
use thiserror::Error;
use xrandr::Monitor as XrandrMonitor;
use xrandr::XHandle;

#[derive(Parser)]
struct XrpexArgs {
    rpex: Rpex<2>,
    #[arg(short, long, env = "XRPEX_MONITOR")]
    monitor: String,
}

#[derive(Error, Debug)]
enum XrpexError {
    #[error("unable to find monitor with given name")]
    NoMonitor,
    #[error(transparent)]
    Xrandr(#[from] xrandr::XrandrError),
    #[error(transparent)]
    XrandrManager(#[from] XrandrManagerError),
    #[error(transparent)]
    ApplyRpexMonitorError(#[from] ApplyRpexMonitorError<XrandrManagerError>),
}

fn main() -> Result<(), XrpexError> {
    let args = XrpexArgs::parse();

    let mut xrandr = XHandle::open()?;

    xrandr.reset_rpex_monitors(&args.monitor)?;

    let monitor = xrandr
        .get_monitors()?
        .find(|RpexMonitor { name, .. }| *name == args.monitor)
        .ok_or(XrpexError::NoMonitor)?;

    xrandr.apply_rpex_monitors(&monitor, args.rpex)?;

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RpexMonitor {
    name: String,
    resolution: HyperRectangle<2>,
}

trait RpexMonitorManager {
    type ManagerError: std::error::Error;

    fn get_monitors(&mut self) -> Result<impl Iterator<Item = RpexMonitor>, Self::ManagerError>;

    fn reset_rpex_monitors(
        &mut self,
        parent_name: &str,
    ) -> Result<Vec<RpexMonitor>, Self::ManagerError>;

    fn apply_rpex_monitors(
        &mut self,
        parent_monitor: &RpexMonitor,
        rpex: Rpex<2>,
    ) -> Result<(), ApplyRpexMonitorError<Self::ManagerError>>;
}

#[derive(Error, Debug)]
enum ApplyRpexMonitorError<E> {
    #[error("monitor manager error: {0}")]
    ManagerError(E),
    #[error("failed to evaluate rpex this monitor: {0}")]
    RpexEvaluation(#[from] SumsInRatioEvaluationError),
}

#[derive(Error, Debug)]
enum XrandrManagerError {
    #[error("encountered xrandr lib error: {0}")]
    Xrandr(#[from] xrandr::XrandrError),
    #[error("encountered io error: {0}")]
    Io(#[from] std::io::Error),
}

impl RpexMonitorManager for XHandle {
    type ManagerError = XrandrManagerError;

    fn get_monitors(&mut self) -> Result<impl Iterator<Item = RpexMonitor>, Self::ManagerError> {
        Ok(self.monitors()?.into_iter().map(
            |XrandrMonitor {
                 name,
                 width_px,
                 height_px,
                 ..
             }| RpexMonitor {
                name,
                resolution: HyperRectangle {
                    lengths: [width_px as u32, height_px as u32],
                },
            },
        ))
    }

    fn reset_rpex_monitors(
        &mut self,
        parent_name: &str,
    ) -> Result<Vec<RpexMonitor>, Self::ManagerError> {
        let monitors_to_delete = self
            .get_monitors()?
            .filter(|RpexMonitor { name, .. }| {
                name.starts_with(format!("{parent_name}-XRPEX").as_str())
            })
            .collect::<Vec<_>>();

        monitors_to_delete
            .iter()
            .fold(
                Command::new("xrandr"),
                |mut command, RpexMonitor { name, .. }| {
                    command.args(["--delmonitor", &name]);
                    command
                },
            )
            .output()?;

        Ok(monitors_to_delete)
    }

    fn apply_rpex_monitors(
        &mut self,
        parent_monitor: &RpexMonitor,
        rpex: Rpex<2>,
    ) -> Result<(), ApplyRpexMonitorError<Self::ManagerError>> {
        let (evaluated, scale) = rpex.evaluate(parent_monitor.resolution)?;

        let parent_name = &parent_monitor.name;

        evaluated
            .iter_partitions()
            .fold(
                Command::new("xrandr"),
                |mut command,
                 Partition {
                     ratio,
                     ratio_position,
                 }| {
                    let [width, height] = ratio.map(|r| r * scale);
                    let [x, y] = ratio_position.map(|r| r * scale);

                    let name = format!("{parent_name}-XRPEX-{x}-{y}",);

                    let geometry = format!("{width}/0x{height}/1+{x}+{y}");

                    command.args(["--setmonitor", &name, &geometry, &parent_name]);
                    command
                },
            )
            .output()
            .map_err(|e| ApplyRpexMonitorError::ManagerError(XrandrManagerError::Io(e)))?;

        Ok(())
    }
}
