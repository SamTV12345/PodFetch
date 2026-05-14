use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::audiobookshelf::book::{Series, SeriesRepository};
use uuid::Uuid;

diesel::table! {
    audiobookshelf_series (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    audiobookshelf_book_series (book_id, series_id) {
        book_id -> Text,
        series_id -> Text,
        sequence -> Nullable<Text>,
    }
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = audiobookshelf_series)]
struct SeriesEntity {
    id: String,
    name: String,
    description: Option<String>,
}

impl From<SeriesEntity> for Series {
    fn from(v: SeriesEntity) -> Self {
        Self {
            id: v.id,
            name: v.name,
            description: v.description,
        }
    }
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = audiobookshelf_book_series)]
struct BookSeriesEntity {
    book_id: String,
    series_id: String,
    sequence: Option<String>,
}

pub struct DieselSeriesRepository {
    database: Database,
}

impl DieselSeriesRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl SeriesRepository for DieselSeriesRepository {
    type Error = PersistenceError;

    fn upsert_by_name(&self, lookup_name: &str) -> Result<Series, Self::Error> {
        use self::audiobookshelf_series::dsl::*;
        let mut conn = self.database.connection()?;
        if let Some(existing) = audiobookshelf_series
            .filter(name.eq(lookup_name))
            .first::<SeriesEntity>(&mut conn)
            .optional()
            .map_err(PersistenceError::from)?
        {
            return Ok(existing.into());
        }
        let entity = SeriesEntity {
            id: format!("ser_{}", Uuid::new_v4().simple()),
            name: lookup_name.to_string(),
            description: None,
        };
        diesel::insert_into(audiobookshelf_series)
            .values(entity.clone())
            .execute(&mut conn)
            .map_err(PersistenceError::from)?;
        Ok(entity.into())
    }

    fn list_for_book(
        &self,
        lookup_book_id: &str,
    ) -> Result<Vec<(Series, Option<String>)>, Self::Error> {
        use self::audiobookshelf_book_series::dsl as bs_dsl;
        use self::audiobookshelf_series::dsl as ser_dsl;
        let mut conn = self.database.connection()?;
        let links: Vec<BookSeriesEntity> = bs_dsl::audiobookshelf_book_series
            .filter(bs_dsl::book_id.eq(lookup_book_id))
            .load::<BookSeriesEntity>(&mut conn)
            .map_err(PersistenceError::from)?;
        if links.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<String> = links.iter().map(|l| l.series_id.clone()).collect();
        let series: Vec<SeriesEntity> = ser_dsl::audiobookshelf_series
            .filter(ser_dsl::id.eq_any(&ids))
            .load::<SeriesEntity>(&mut conn)
            .map_err(PersistenceError::from)?;
        let mut out = Vec::with_capacity(links.len());
        for link in links {
            if let Some(s) = series.iter().find(|s| s.id == link.series_id) {
                out.push((Series::from(s.clone()), link.sequence));
            }
        }
        Ok(out)
    }

    fn link(
        &self,
        book_id_value: &str,
        series_id_value: &str,
        sequence_value: Option<&str>,
    ) -> Result<(), Self::Error> {
        use self::audiobookshelf_book_series::dsl::*;
        let mut conn = self.database.connection()?;
        let exists = audiobookshelf_book_series
            .filter(book_id.eq(book_id_value))
            .filter(series_id.eq(series_id_value))
            .first::<BookSeriesEntity>(&mut conn)
            .optional()
            .map_err(PersistenceError::from)?
            .is_some();
        if exists {
            diesel::update(
                audiobookshelf_book_series
                    .filter(book_id.eq(book_id_value))
                    .filter(series_id.eq(series_id_value)),
            )
            .set(sequence.eq(sequence_value.map(|s| s.to_string())))
            .execute(&mut conn)
            .map_err(PersistenceError::from)?;
        } else {
            diesel::insert_into(audiobookshelf_book_series)
                .values(BookSeriesEntity {
                    book_id: book_id_value.to_string(),
                    series_id: series_id_value.to_string(),
                    sequence: sequence_value.map(|s| s.to_string()),
                })
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        }
        Ok(())
    }

    fn unlink_all_for_book(&self, lookup_book_id: &str) -> Result<usize, Self::Error> {
        use self::audiobookshelf_book_series::dsl::*;
        let mut conn = self.database.connection()?;
        diesel::delete(audiobookshelf_book_series.filter(book_id.eq(lookup_book_id)))
            .execute(&mut conn)
            .map_err(Into::into)
    }
}
