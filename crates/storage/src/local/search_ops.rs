//! Local store search and delete operations

use super::types::LocalSearchResult;
use super::store::LocalStore;
use super::super::client::{SearchOptions, SearchFilter};
use anyhow::Result as AnyResult;

impl LocalStore {
    /// Search for similar vectors using cosine similarity
    pub fn search(&self, vector: &[f32], options: SearchOptions) -> AnyResult<Vec<LocalSearchResult>> {
        let points = self.points.read().unwrap();
        let mut results: Vec<LocalSearchResult> = points.iter()
            .filter(|p| {
                if p.vector.len() != vector.len() {
                    return false;
                }
                if let Some(ref filter) = options.filter {
                    if let Some(ref lang) = filter.language {
                        if p.payload.language != *lang { return false; }
                    }
                    if let Some(ref file) = filter.file {
                        if !p.payload.file.contains(file) { return false; }
                    }
                    if let Some(ref kind) = filter.kind {
                        if p.payload.kind != *kind { return false; }
                    }
                    if let Some(ref repo) = filter.repo {
                        if p.payload.repo != *repo { return false; }
                    }
                }
                true
            })
            .map(|p| {
                let score = super::store::cosine_similarity_fn(&p.vector, vector);
                LocalSearchResult { id: p.id.clone(), score, payload: p.payload.clone() }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(threshold) = options.score_threshold {
            results.retain(|r| r.score >= threshold);
        }

        let offset = options.offset.unwrap_or(0);
        let limit = if options.limit > 0 { options.limit } else { results.len().saturating_sub(offset) };
        results = results.into_iter().skip(offset).take(limit).collect();

        Ok(results)
    }

    /// Delete a point by id
    pub fn delete(&self, id: &str) -> AnyResult<()> {
        let mut points = self.points.write().unwrap();
        points.retain(|p| p.id != id);
        let mut dirty = self.dirty.write().unwrap();
        *dirty = true;
        Ok(())
    }

    /// Delete points matching filter
    pub fn delete_by_filter(&self, filter: &SearchFilter) -> AnyResult<()> {
        let mut points = self.points.write().unwrap();
        points.retain(|p| {
            let mut matches = true;
            if let Some(ref lang) = filter.language {
                if p.payload.language != *lang { matches = false; }
            }
            if let Some(ref file) = filter.file {
                if !p.payload.file.contains(file) { matches = false; }
            }
            if let Some(ref kind) = filter.kind {
                if p.payload.kind != *kind { matches = false; }
            }
            if let Some(ref repo) = filter.repo {
                if p.payload.repo != *repo { matches = false; }
            }
            !matches
        });
        let mut dirty = self.dirty.write().unwrap();
        *dirty = true;
        Ok(())
    }
}
