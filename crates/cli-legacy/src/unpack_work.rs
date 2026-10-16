use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use kfc::container::KFCFile;
use kfc::guid::ResourceId;

/// Shared work queue for unpacking resources in file order.
///
/// Workers claim contiguous batches instead of taking a lock for every resource.
/// Several batches per worker keep expensive descriptors from stalling the export,
/// while nearby resources reuse the same decompressed chunks.
pub(crate) struct UnpackWork<'a> {
    guids: Vec<&'a ResourceId>,
    batch_size: usize,
    next: AtomicUsize,
}

impl<'a> UnpackWork<'a> {
    pub(crate) fn new(file: &KFCFile, guids: HashSet<&'a ResourceId>, thread_count: u8) -> Self {
        Self::from_offsets(
            guids.into_iter().map(|guid| {
                let offset = file
                    .resources()
                    .get(guid)
                    .map(|entry| entry.offset)
                    .unwrap_or(u64::MAX);
                (guid, offset)
            }),
            thread_count,
        )
    }

    fn from_offsets(
        resources: impl Iterator<Item = (&'a ResourceId, u64)>,
        thread_count: u8,
    ) -> Self {
        let mut resources = resources.collect::<Vec<_>>();
        resources.sort_unstable_by_key(|&(guid, offset)| (offset, *guid));

        // Bound batch size so one large chunk can still use several workers.
        let batch_count = usize::from(thread_count).max(1) * 8;
        let batch_size = resources.len().div_ceil(batch_count).clamp(1, 256);

        Self {
            guids: resources.into_iter().map(|(guid, _)| guid).collect(),
            batch_size,
            next: AtomicUsize::new(0),
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &'a ResourceId> + '_ {
        std::iter::from_fn(|| {
            let start = self.next.fetch_add(self.batch_size, Ordering::Relaxed);
            let end = start.saturating_add(self.batch_size).min(self.guids.len());
            self.guids.get(start..end)
        })
        .flatten()
        .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guids(count: u32) -> Vec<ResourceId> {
        (0..count)
            .map(|index| ResourceId::NONE.with_part_index(index))
            .collect()
    }

    #[test]
    fn resources_are_processed_in_offset_order() {
        let guids = guids(4);
        let work = UnpackWork::from_offsets(
            [
                (&guids[0], 900),
                (&guids[1], 16),
                (&guids[2], 512),
                (&guids[3], 16),
            ]
            .into_iter(),
            1,
        );

        assert_eq!(
            work.iter()
                .map(|guid| guid.part_index())
                .collect::<Vec<_>>(),
            [1, 3, 2, 0],
        );
    }

    #[test]
    fn parallel_workers_process_every_resource_once_in_ascending_order() {
        let guids = guids(10_003);
        let work = UnpackWork::from_offsets(
            guids
                .iter()
                .rev()
                .map(|guid| (guid, u64::from(guid.part_index()) * 16)),
            8,
        );
        let start = std::sync::Barrier::new(8);

        let mut processed = std::thread::scope(|scope| {
            let handles = (0..8)
                .map(|_| {
                    let work = &work;
                    let start = &start;
                    scope.spawn(move || {
                        start.wait();
                        let processed = work
                            .iter()
                            .map(|guid| guid.part_index())
                            .collect::<Vec<_>>();
                        assert!(processed.windows(2).all(|pair| pair[0] < pair[1]));
                        processed
                    })
                })
                .collect::<Vec<_>>();

            handles
                .into_iter()
                .flat_map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });

        processed.sort_unstable();
        assert_eq!(processed, (0..10_003).collect::<Vec<_>>());
        assert_eq!(work.iter().count(), 0);
    }

    #[test]
    fn empty_and_small_selections_are_not_lost() {
        let empty = UnpackWork::new(&KFCFile::default(), HashSet::new(), 8);
        assert_eq!(empty.iter().count(), 0);

        let guids = guids(1);
        // Unknown IDs must reach the reader's existing error handling.
        let work = UnpackWork::new(&KFCFile::default(), guids.iter().collect(), 8);
        assert_eq!(work.iter().collect::<Vec<_>>(), [&guids[0]]);
        assert_eq!(work.iter().count(), 0);
    }

    #[test]
    fn worker_only_claims_work_when_it_starts_processing() {
        let guids = guids(7);
        let work = UnpackWork::from_offsets(guids.iter().map(|guid| (guid, 0)), 8);
        let unused_worker = work.iter();
        drop(unused_worker);

        assert_eq!(work.iter().count(), guids.len());
    }
}
