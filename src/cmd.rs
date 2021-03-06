//! Ceph has a command system defined
//! in https://github.com/ceph/ceph/blob/master/src/mon/MonCommands.h
//! The cli commands mostly use this json based system.  This allows you to
//! make the exact
//! same calls without having to shell out with std::process::Command.
//! Many of the commands defined in this file have a simulate parameter to
//! allow you to test without actually calling Ceph.
extern crate serde_json;

use ceph::ceph_mon_command_without_data;
use error::{RadosError, RadosResult};
use rados::rados_t;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct CephMon {
    pub rank: i64,
    pub name: String,
    pub addr: String,
}

#[derive(Deserialize, Debug)]
pub struct CrushNode {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub crush_type: String,
    pub type_id: i64,
    pub children: Option<Vec<i64>>,
    pub crush_weight: Option<f64>,
    pub depth: Option<i64>,
    pub exists: Option<i64>,
    pub status: Option<String>,
    pub reweight: Option<f64>,
    pub primary_affinity: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct CrushTree {
    pub nodes: Vec<CrushNode>,
    pub stray: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct MgrMetadata {
    pub id: String,
    pub arch: String,
    pub ceph_version: String,
    pub cpu: String,
    pub distro: String,
    pub distro_description: String,
    pub distro_version: String,
    pub hostname: String,
    pub kernel_description: String,
    pub kernel_version: String,
    pub mem_swap_kb: u64,
    pub mem_total_kb: u64,
    pub os: String,
}

#[derive(Deserialize, Debug)]
pub struct MgrStandby {
    pub gid: u64,
    pub name: String,
    pub available_modules: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct MgrDump {
    pub epoch: u64,
    pub active_gid: u64,
    pub active_name: String,
    pub active_addr: String,
    pub available: bool,
    pub standbys: Vec<MgrStandby>,
    pub modules: Vec<String>,
    pub available_modules: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct MonDump {
    pub epoch: i64,
    pub fsid: String,
    pub modified: String,
    pub created: String,
    pub mons: Vec<CephMon>,
    pub quorum: Vec<i64>,
}

#[derive(Deserialize, Debug)]
pub struct MonStatus {
    pub name: String,
    pub rank: u64,
    pub state: MonState,
    pub election_epoch: u64,
    pub quorum: Vec<u64>,
    pub outside_quorum: Vec<u64>,
    pub extra_probe_peers: Vec<u64>,
    pub sync_provider: Vec<u64>,
    pub monmap: MonMap,
}

#[derive(Deserialize, Debug)]
pub struct MonMap {
    pub epoch: u64,
    pub fsid: Uuid,
    pub modified: String,
    pub created: String,
    pub mons: Vec<Mon>,
}

#[derive(Deserialize, Debug)]
pub struct Mon {
    pub rank: u64,
    pub name: String,
    pub addr: String,
}

#[derive(Deserialize, Debug)]
pub enum HealthStatus {
    #[serde(rename = "HEALTH_ERR")]
    Err,
    #[serde(rename = "HEALTH_WARN")]
    Warn,
    #[serde(rename = "HEALTH_OK")]
    Ok,
}

#[derive(Deserialize, Debug)]
pub struct ClusterHealth {
    pub health: Health,
    pub timechecks: TimeChecks,
    pub summary: Vec<String>,
    pub overall_status: HealthStatus,
    pub detail: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Health {
    pub health_services: Vec<ServiceHealth>,
}

#[derive(Deserialize, Debug)]
pub struct TimeChecks {
    pub epoch: u64,
    pub round: u64,
    pub round_status: RoundStatus,
    pub mons: Vec<MonTimeChecks>,
}

#[derive(Deserialize, Debug)]
pub struct MonTimeChecks {
    pub name: String,
    pub skew: f64,
    pub latency: f64,
    pub health: HealthStatus,
}

#[derive(Deserialize, Debug)]
pub struct ServiceHealth {
    pub mons: Vec<MonHealth>,
}

#[derive(Deserialize, Debug)]
pub struct MonHealth {
    pub name: String,
    pub kb_total: u64,
    pub kb_used: u64,
    pub kb_avail: u64,
    pub avail_percent: u8,
    pub last_updated: String,
    pub store_stats: StoreStats,
    pub health: HealthStatus,
}

#[derive(Deserialize, Debug)]
pub struct StoreStats {
    pub bytes_total: u64,
    pub bytes_sst: u64,
    pub bytes_log: u64,
    pub bytes_misc: u64,
    pub last_updated: String,
}

#[derive(Deserialize, Debug)]
pub enum RoundStatus {
    #[serde(rename = "finished")]
    Finished,
    #[serde(rename = "on-going")]
    OnGoing,
}

#[derive(Deserialize, Debug)]
pub enum MonState {
    #[serde(rename = "probing")]
    Probing,
    #[serde(rename = "synchronizing")]
    Synchronizing,
    #[serde(rename = "electing")]
    Electing,
    #[serde(rename = "leader")]
    Leader,
    #[serde(rename = "peon")]
    Peon,
    #[serde(rename = "shutdown")]
    Shutdown,
}

#[derive(Deserialize, Debug, Serialize)]
pub enum OsdOption {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "noup")]
    NoUp,
    #[serde(rename = "nodown")]
    NoDown,
    #[serde(rename = "noout")]
    NoOut,
    #[serde(rename = "noin")]
    NoIn,
    #[serde(rename = "nobackfill")]
    NoBackfill,
    #[serde(rename = "norebalance")]
    NoRebalance,
    #[serde(rename = "norecover")]
    NoRecover,
    #[serde(rename = "noscrub")]
    NoScrub,
    #[serde(rename = "nodeep-scrub")]
    NoDeepScrub,
    #[serde(rename = "notieragent")]
    NoTierAgent,
    #[serde(rename = "sortbitwise")]
    SortBitwise,
    #[serde(rename = "recovery_deletes")]
    RecoveryDeletes,
    #[serde(rename = "require_jewel_osds")]
    RequireJewelOsds,
    #[serde(rename = "require_kraken_osds")]
    RequireKrakenOsds,
}

impl fmt::Display for OsdOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &OsdOption::Full => write!(f, "full"),
            &OsdOption::Pause => write!(f, "pause"),
            &OsdOption::NoUp => write!(f, "noup"),
            &OsdOption::NoDown => write!(f, "nodown"),
            &OsdOption::NoOut => write!(f, "noout"),
            &OsdOption::NoIn => write!(f, "noin"),
            &OsdOption::NoBackfill => write!(f, "nobackfill"),
            &OsdOption::NoRebalance => write!(f, "norebalance"),
            &OsdOption::NoRecover => write!(f, "norecover"),
            &OsdOption::NoScrub => write!(f, "noscrub"),
            &OsdOption::NoDeepScrub => write!(f, "nodeep-scrub"),
            &OsdOption::NoTierAgent => write!(f, "notieragent"),
            &OsdOption::SortBitwise => write!(f, "sortbitwise"),
            &OsdOption::RecoveryDeletes => write!(f, "recovery_deletes"),
            &OsdOption::RequireJewelOsds => write!(f, "require_jewel_osds"),
            &OsdOption::RequireKrakenOsds => write!(f, "require_kraken_osds"),
        }
    }
}

impl AsRef<str> for OsdOption {
    fn as_ref(&self) -> &str {
        match self {
            &OsdOption::Full => "full",
            &OsdOption::Pause => "pause",
            &OsdOption::NoUp => "noup",
            &OsdOption::NoDown => "nodown",
            &OsdOption::NoOut => "noout",
            &OsdOption::NoIn => "noin",
            &OsdOption::NoBackfill => "nobackfill",
            &OsdOption::NoRebalance => "norebalance",
            &OsdOption::NoRecover => "norecover",
            &OsdOption::NoScrub => "noscrub",
            &OsdOption::NoDeepScrub => "nodeep-scrub",
            &OsdOption::NoTierAgent => "notieragent",
            &OsdOption::SortBitwise => "sortbitwise",
            &OsdOption::RecoveryDeletes => "recovery_deletes",
            &OsdOption::RequireJewelOsds => "require_jewel_osds",
            &OsdOption::RequireKrakenOsds => "require_kraken_osds",
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub enum PoolOption {
    #[serde(rename = "size")]
    Size,
    #[serde(rename = "min_size")]
    MinSize,
    #[serde(rename = "crash_replay_interval")]
    CrashReplayInterval,
    #[serde(rename = "pg_num")]
    PgNum,
    #[serde(rename = "pgp_num")]
    PgpNum,
    #[serde(rename = "crush_rule")]
    CrushRule,
    #[serde(rename = "hashpspool")]
    HashPsPool,
    #[serde(rename = "nodelete")]
    NoDelete,
    #[serde(rename = "nopgchange")]
    NoPgChange,
    #[serde(rename = "nosizechange")]
    NoSizeChange,
    #[serde(rename = "write_fadvice_dontneed")]
    WriteFadviceDontNeed,
    #[serde(rename = "noscrub")]
    NoScrub,
    #[serde(rename = "nodeep-scrub")]
    NoDeepScrub,
    #[serde(rename = "hit_set_type")]
    HitSetType,
    #[serde(rename = "hit_set_period")]
    HitSetPeriod,
    #[serde(rename = "hit_set_count")]
    HitSetCount,
    #[serde(rename = "hit_set_fpp")]
    HitSetFpp,
    #[serde(rename = "use_gmt_hitset")]
    UseGmtHitset,
    #[serde(rename = "target_max_bytes")]
    TargetMaxBytes,
    #[serde(rename = "target_max_objects")]
    TargetMaxObjects,
    #[serde(rename = "cache_target_dirty_ratio")]
    CacheTargetDirtyRatio,
    #[serde(rename = "cache_target_dirty_high_ratio")]
    CacheTargetDirtyHighRatio,
    #[serde(rename = "cache_target_full_ratio")]
    CacheTargetFullRatio,
    #[serde(rename = "cache_min_flush_age")]
    CacheMinFlushAge,
    #[serde(rename = "cachem_min_evict_age")]
    CacheMinEvictAge,
    #[serde(rename = "auid")]
    Auid,
    #[serde(rename = "min_read_recency_for_promote")]
    MinReadRecencyForPromote,
    #[serde(rename = "min_write_recency_for_promote")]
    MinWriteRecencyForPromte,
    #[serde(rename = "fast_read")]
    FastRead,
    #[serde(rename = "hit_set_decay_rate")]
    HitSetGradeDecayRate,
    #[serde(rename = "hit_set_search_last_n")]
    HitSetSearchLastN,
    #[serde(rename = "scrub_min_interval")]
    ScrubMinInterval,
    #[serde(rename = "scrub_max_interval")]
    ScrubMaxInterval,
    #[serde(rename = "deep_scrub_interval")]
    DeepScrubInterval,
    #[serde(rename = "recovery_priority")]
    RecoveryPriority,
    #[serde(rename = "recovery_op_priority")]
    RecoveryOpPriority,
    #[serde(rename = "scrub_priority")]
    ScrubPriority,
    #[serde(rename = "compression_mode")]
    CompressionMode,
    #[serde(rename = "compression_algorithm")]
    CompressionAlgorithm,
    #[serde(rename = "compression_required_ratio")]
    CompressionRequiredRatio,
    #[serde(rename = "compression_max_blob_size")]
    CompressionMaxBlobSize,
    #[serde(rename = "compression_min_blob_size")]
    CompressionMinBlobSize,
    #[serde(rename = "csum_type")]
    CsumType,
    #[serde(rename = "csum_min_block")]
    CsumMinBlock,
    #[serde(rename = "csum_max_block")]
    CsumMaxBlock,
    #[serde(rename = "allow_ec_overwrites")]
    AllocEcOverwrites,
}

impl fmt::Display for PoolOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &PoolOption::Size => write!(f, "size"),
            &PoolOption::MinSize => write!(f, "min_size"),
            &PoolOption::CrashReplayInterval => write!(f, "crash_replay_interval"),
            &PoolOption::PgNum => write!(f, "pg_num"),
            &PoolOption::PgpNum => write!(f, "pgp_num"),
            &PoolOption::CrushRule => write!(f, "crush_rule"),
            &PoolOption::HashPsPool => write!(f, "hashpspool"),
            &PoolOption::NoDelete => write!(f, "nodelete"),
            &PoolOption::NoPgChange => write!(f, "nopgchange"),
            &PoolOption::NoSizeChange => write!(f, "nosizechange"),
            &PoolOption::WriteFadviceDontNeed => write!(f, "write_fadvice_dontneed"),
            &PoolOption::NoScrub => write!(f, "noscrub"),
            &PoolOption::NoDeepScrub => write!(f, "nodeep-scrub"),
            &PoolOption::HitSetType => write!(f, "hit_set_type"),
            &PoolOption::HitSetPeriod => write!(f, "hit_set_period"),
            &PoolOption::HitSetCount => write!(f, "hit_set_count"),
            &PoolOption::HitSetFpp => write!(f, "hit_set_fpp"),
            &PoolOption::UseGmtHitset => write!(f, "use_gmt_hitset"),
            &PoolOption::TargetMaxBytes => write!(f, "target_max_bytes"),
            &PoolOption::TargetMaxObjects => write!(f, "target_max_objects"),
            &PoolOption::CacheTargetDirtyRatio => write!(f, "cache_target_dirty_ratio"),
            &PoolOption::CacheTargetDirtyHighRatio => write!(f, "cache_target_dirty_high_ratio"),
            &PoolOption::CacheTargetFullRatio => write!(f, "cache_target_full_ratio"),
            &PoolOption::CacheMinFlushAge => write!(f, "cache_min_flush_age"),
            &PoolOption::CacheMinEvictAge => write!(f, "cachem_min_evict_age"),
            &PoolOption::Auid => write!(f, "auid"),
            &PoolOption::MinReadRecencyForPromote => write!(f, "min_read_recency_for_promote"),
            &PoolOption::MinWriteRecencyForPromte => write!(f, "min_write_recency_for_promote"),
            &PoolOption::FastRead => write!(f, "fast_read"),
            &PoolOption::HitSetGradeDecayRate => write!(f, "hit_set_decay_rate"),
            &PoolOption::HitSetSearchLastN => write!(f, "hit_set_search_last_n"),
            &PoolOption::ScrubMinInterval => write!(f, "scrub_min_interval"),
            &PoolOption::ScrubMaxInterval => write!(f, "scrub_max_interval"),
            &PoolOption::DeepScrubInterval => write!(f, "deep_scrub_interval"),
            &PoolOption::RecoveryPriority => write!(f, "recovery_priority"),
            &PoolOption::RecoveryOpPriority => write!(f, "recovery_op_priority"),
            &PoolOption::ScrubPriority => write!(f, "scrub_priority"),
            &PoolOption::CompressionMode => write!(f, "compression_mode"),
            &PoolOption::CompressionAlgorithm => write!(f, "compression_algorithm"),
            &PoolOption::CompressionRequiredRatio => write!(f, "compression_required_ratio"),
            &PoolOption::CompressionMaxBlobSize => write!(f, "compression_max_blob_size"),
            &PoolOption::CompressionMinBlobSize => write!(f, "compression_min_blob_size"),
            &PoolOption::CsumType => write!(f, "csum_type"),
            &PoolOption::CsumMinBlock => write!(f, "csum_min_block"),
            &PoolOption::CsumMaxBlock => write!(f, "csum_max_block"),
            &PoolOption::AllocEcOverwrites => write!(f, "allow_ec_overwrites"),
        }
    }
}

impl AsRef<str> for PoolOption {
    fn as_ref(&self) -> &str {
        match self {
            &PoolOption::Size => "size",
            &PoolOption::MinSize => "min_size",
            &PoolOption::CrashReplayInterval => "crash_replay_interval",
            &PoolOption::PgNum => "pg_num",
            &PoolOption::PgpNum => "pgp_num",
            &PoolOption::CrushRule => "crush_rule",
            &PoolOption::HashPsPool => "hashpspool",
            &PoolOption::NoDelete => "nodelete",
            &PoolOption::NoPgChange => "nopgchange",
            &PoolOption::NoSizeChange => "nosizechange",
            &PoolOption::WriteFadviceDontNeed => "write_fadvice_dontneed",
            &PoolOption::NoScrub => "noscrub",
            &PoolOption::NoDeepScrub => "nodeep-scrub",
            &PoolOption::HitSetType => "hit_set_type",
            &PoolOption::HitSetPeriod => "hit_set_period",
            &PoolOption::HitSetCount => "hit_set_count",
            &PoolOption::HitSetFpp => "hit_set_fpp",
            &PoolOption::UseGmtHitset => "use_gmt_hitset",
            &PoolOption::TargetMaxBytes => "target_max_bytes",
            &PoolOption::TargetMaxObjects => "target_max_objects",
            &PoolOption::CacheTargetDirtyRatio => "cache_target_dirty_ratio",
            &PoolOption::CacheTargetDirtyHighRatio => "cache_target_dirty_high_ratio",
            &PoolOption::CacheTargetFullRatio => "cache_target_full_ratio",
            &PoolOption::CacheMinFlushAge => "cache_min_flush_age",
            &PoolOption::CacheMinEvictAge => "cachem_min_evict_age",
            &PoolOption::Auid => "auid",
            &PoolOption::MinReadRecencyForPromote => "min_read_recency_for_promote",
            &PoolOption::MinWriteRecencyForPromte => "min_write_recency_for_promote",
            &PoolOption::FastRead => "fast_read",
            &PoolOption::HitSetGradeDecayRate => "hit_set_decay_rate",
            &PoolOption::HitSetSearchLastN => "hit_set_search_last_n",
            &PoolOption::ScrubMinInterval => "scrub_min_interval",
            &PoolOption::ScrubMaxInterval => "scrub_max_interval",
            &PoolOption::DeepScrubInterval => "deep_scrub_interval",
            &PoolOption::RecoveryPriority => "recovery_priority",
            &PoolOption::RecoveryOpPriority => "recovery_op_priority",
            &PoolOption::ScrubPriority => "scrub_priority",
            &PoolOption::CompressionMode => "compression_mode",
            &PoolOption::CompressionAlgorithm => "compression_algorithm",
            &PoolOption::CompressionRequiredRatio => "compression_required_ratio",
            &PoolOption::CompressionMaxBlobSize => "compression_max_blob_size",
            &PoolOption::CompressionMinBlobSize => "compression_min_blob_size",
            &PoolOption::CsumType => "csum_type",
            &PoolOption::CsumMinBlock => "csum_min_block",
            &PoolOption::CsumMaxBlock => "csum_max_block",
            &PoolOption::AllocEcOverwrites => "allow_ec_overwrites",
        }
    }
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &HealthStatus::Err => write!(f, "HEALTH_ERR"),
            &HealthStatus::Ok => write!(f, "HEALTH_OK"),
            &HealthStatus::Warn => write!(f, "HEALTH_WARN"),
        }
    }
}

impl AsRef<str> for HealthStatus {
    fn as_ref(&self) -> &str {
        match self {
            &HealthStatus::Err => "HEALTH_ERR",
            &HealthStatus::Ok => "HEALTH_OK",
            &HealthStatus::Warn => "HEALTH_WARN",
        }
    }
}

impl fmt::Display for RoundStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &RoundStatus::Finished => write!(f, "finished"),
            &RoundStatus::OnGoing => write!(f, "on-going"),
        }
    }
}

impl AsRef<str> for RoundStatus {
    fn as_ref(&self) -> &str {
        match self {
            &RoundStatus::Finished => "finished",
            &RoundStatus::OnGoing => "on-going",
        }
    }
}

pub fn cluster_health(cluster_handle: rados_t) -> RadosResult<ClusterHealth> {
    let cmd = json!({
        "prefix": "health",
    });
    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse health output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(result.1.unwrap_or(
        "No response from ceph for health".into(),
    )))
}

pub fn osd_out(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd out",
        "ids": [osd_id.to_string()]
    });

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

pub fn osd_crush_remove(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd crush remove",
        "name": format!("osd.{}", osd_id),
    });
    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

/// Query a ceph pool.
pub fn osd_pool_get(cluster_handle: rados_t, pool: &str, choice: &PoolOption) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "osd pool get",
        "pool": pool,
        "var": choice,
    });
    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(res.into()),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse osd pool get output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(result.1.unwrap_or(
        "No response from ceph for osd pool get".into(),
    )))
}

/// Set a pool value
pub fn osd_pool_set(cluster_handle: rados_t, pool: &str, key: &PoolOption, value: &str, simulate: bool)
    -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd pool set",
        "pool": pool,
        "var": key,
        "val": value,
    });
    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

pub fn osd_set(cluster_handle: rados_t, key: &OsdOption, force: bool, simulate: bool) -> RadosResult<()> {
    let cmd = match force {
        true => {
            json!({
                "prefix": "osd set",
                "key": key,
                "sure": "--yes-i-really-mean-it",
            })
        },
        false => {
            json!({
                "prefix": "osd set",
                "key": key,
            })
        },
    };
    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

pub fn osd_unset(cluster_handle: rados_t, key: &OsdOption, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd unset",
        "key": key,
    });
    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

pub fn osd_tree(cluster_handle: rados_t) -> RadosResult<CrushTree> {
    let cmd = json!({
        "prefix": "osd tree",
        "format": "json"
    });
    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse osd tree output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for osd tree".into()))
}

// Get cluster status
pub fn status(cluster_handle: rados_t) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "status",
        "format": "json"
    });
    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(res.into()),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse status output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for status".into()))
}

/// List all the monitors in the cluster and their current rank
pub fn mon_dump(cluster_handle: rados_t) -> RadosResult<MonDump> {
    let cmd = json!({
        "prefix": "mon dump",
        "format": "json"
    });
    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mon dump output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for mon dump".into()))
}

/// Get the mon quorum
pub fn mon_quorum(cluster_handle: rados_t) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "quorum_status",
        "format": "json"
    });
    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse quorum_status output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for quorum_status".into()))
}

/// Get the mon status
pub fn mon_status(cluster_handle: rados_t) -> RadosResult<MonStatus> {
    let cmd = json!({
        "prefix": "mon_status",
    });
    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mon_status output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for mon_status".into()))
}

/// Show mon daemon version
pub fn version(cluster_handle: rados_t) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "version",
    });
    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(res.to_string()),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse version output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for version".into()))
}


pub fn osd_pool_quota_get(cluster_handle: rados_t, pool: &str) -> RadosResult<u64> {
    let cmd = json!({
        "prefix": "osd pool get-quota",
        "pool": pool
    });
    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(u64::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse osd pool quota-get output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error("No response from ceph for osd pool quota-get".into()))
}

pub fn auth_del(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "auth del",
        "entity": format!("osd.{}", osd_id)
    });

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

pub fn osd_rm(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd rm",
        "ids": [osd_id.to_string()]
    });

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())

}

pub fn osd_create(cluster_handle: rados_t, id: Option<u64>, simulate: bool) -> RadosResult<u64> {
    let cmd = match id {
        Some(osd_id) => {
            json!({
                "prefix": "osd create",
                "id": format!("osd.{}", osd_id),
            })
        },
        None => {
            json!({
                "prefix": "osd create"
            })
        },
    };

    if simulate {
        return Ok(0);
    }

    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(num) => return Ok(u64::from_str(num)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse osd create output: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse osd create output: {:?}", result)))
}

// Add a new mgr to the cluster
pub fn mgr_auth_add(cluster_handle: rados_t, mgr_id: &str, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "auth add",
        "entity": format!("mgr.{}", mgr_id),
        "caps": ["mon", "allow profile mgr", "osd", "allow *", "mds", "allow *"],
    });

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

// Add a new osd to the cluster
pub fn osd_auth_add(cluster_handle: rados_t, osd_id: u64, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "auth add",
        "entity": format!("osd.{}", osd_id),
        "caps": ["mon", "allow rwx", "osd", "allow *"],
    });

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

/// Get a ceph-x key.  The id parameter can be either a number or a string
/// depending on the type of client so I went with string.
pub fn auth_get_key(cluster_handle: rados_t, client_type: &str, id: &str) -> RadosResult<String> {
    let cmd = json!({
        "prefix": "auth get-key",
        "entity": format!("{}.{}", client_type, id),
    });

    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(key) => return Ok(key.into()),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse auth get-key: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse auth get-key output: {:?}", result)))
}

// ceph osd crush add {id-or-name} {weight}  [{bucket-type}={bucket-name} ...]
/// add or update crushmap position and weight for an osd
pub fn osd_crush_add(cluster_handle: rados_t, osd_id: u64, weight: f64, host: &str, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "osd crush add",
        "id": osd_id,
        "weight": weight,
        "args": [format!("host={}", host)]
    });

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

// Luminous mgr commands below

/// dump the latest MgrMap
pub fn mgr_dump(cluster_handle: rados_t) -> RadosResult<MgrDump> {
    let cmd = json!({
        "prefix": "mgr dump",
    });

    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr dump: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr dump output: {:?}", result)))
}

/// Treat the named manager daemon as failed
pub fn mgr_fail(cluster_handle: rados_t, mgr_id: &str, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "mgr fail",
        "name": mgr_id,
    });

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

/// List active mgr modules
pub fn mgr_list_modules(cluster_handle: rados_t) -> RadosResult<Vec<String>> {
    let cmd = json!({
        "prefix": "mgr module ls",
    });

    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr module ls: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr ls output: {:?}", result)))
}

/// List service endpoints provided by mgr modules
pub fn mgr_list_services(cluster_handle: rados_t) -> RadosResult<Vec<String>> {
    let cmd = json!({
        "prefix": "mgr services",
    });

    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr services: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr services output: {:?}", result)))
}

/// Enable a mgr module
pub fn mgr_enable_module(cluster_handle: rados_t, module: &str, force: bool, simulate: bool) -> RadosResult<()> {
    let cmd = match force {
        true => {
            json!({
                    "prefix": "mgr module enable",
                    "module": module,
                    "force": "--force",
                })
        },
        false => {
            json!({
                    "prefix": "mgr module enable",
                    "module": module,
                })
        },
    };

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

/// Disable a mgr module
pub fn mgr_disable_module(cluster_handle: rados_t, module: &str, simulate: bool) -> RadosResult<()> {
    let cmd = json!({
        "prefix": "mgr module disable",
        "module": module,
    });

    if !simulate {
        ceph_mon_command_without_data(cluster_handle, &cmd)?;
    }
    Ok(())
}

/// dump metadata for all daemons
pub fn mgr_metadata(cluster_handle: rados_t) -> RadosResult<MgrMetadata> {
    let cmd = json!({
        "prefix": "mgr metadata",
    });

    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr metadata: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr metadata output: {:?}", result)))
}

/// count ceph-mgr daemons by metadata field property
pub fn mgr_count_metadata(cluster_handle: rados_t, property: &str) -> RadosResult<HashMap<String, u64>> {
    let cmd = json!({
        "prefix": "mgr count-metadata",
        "name": property,
    });

    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr count-metadata: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr count-metadata output: {:?}", result)))
}

/// check running versions of ceph-mgr daemons
pub fn mgr_versions(cluster_handle: rados_t) -> RadosResult<HashMap<String, u64>> {
    let cmd = json!({
        "prefix": "mgr versions",
    });

    let result = ceph_mon_command_without_data(cluster_handle, &cmd)?;
    if let Some(return_data) = result.0 {
        let mut l = return_data.lines();
        match l.next() {
            Some(res) => return Ok(serde_json::from_str(res)?),
            None => {
                return Err(RadosError::Error(format!(
                "Unable to parse mgr versions: {:?}",
                return_data,
            )))
            },
        }
    }
    Err(RadosError::Error(format!("Unable to parse mgr versions output: {:?}", result)))
}
