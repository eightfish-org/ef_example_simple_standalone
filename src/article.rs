use anyhow::{anyhow, bail};
use eightfish_derive::EightFishModel;
use eightfish_sdk::{
    EightFishModel, HandlerCRUD, Info, Module, Request, Response, Result, Router, Status,
};
use serde::{Deserialize, Serialize};
use spin_sdk::pg::{self, DbValue, Decode, ParameterValue};

const REDIS_URL_ENV: &str = "REDIS_URL_ENV";
const DB_URL_ENV: &str = "DB_URL_ENV";

#[derive(Debug, Clone, Serialize, Deserialize, EightFishModel, Default)]
pub struct Article {
    id: String,
    title: String,
    content: String,
    authorname: String,
}

pub struct ArticleModule;

impl ArticleModule {
    fn get_one(req: &mut Request) -> Result<Response> {
        let pg_addr = std::env::var(DB_URL_ENV)?;
        let pg_conn = pg::Connection::open(&pg_addr)?;

        let params = req.parse_urlencoded()?;
        let article_id = params.get("id").ok_or(anyhow!("id error"))?;

        let (sql, sql_params) = Article::build_get_by_id(article_id);
        let rowset = pg_conn.query(&sql, &sql_params)?;

        let results = if let Some(row) = rowset.rows.into_iter().next() {
            vec![Article::from_row(row)]
        } else {
            bail!("no this item".to_string());
        };

        let info = Info {
            model_name: Article::model_name(),
            action: HandlerCRUD::GetOne,
            extra: "".to_string(),
        };

        Ok(Response::new(Status::Successful, info, results))
    }

    fn new_article(req: &mut Request) -> Result<Response> {
        let pg_addr = std::env::var(DB_URL_ENV)?;
        let pg_conn = pg::Connection::open(&pg_addr)?;

        let params = req.parse_urlencoded()?;
        let title = params
            .get("title")
            .ok_or(anyhow!("title error"))?
            .to_owned();
        let content = params
            .get("content")
            .ok_or(anyhow!("content error"))?
            .to_owned();
        let authorname = params
            .get("authorname")
            .ok_or(anyhow!("authorname error"))?
            .to_owned();
        let id = req
            .ext()
            .get("random_str")
            .ok_or(anyhow!("id error"))?
            .to_owned();

        let article = Article {
            id,
            title,
            content,
            authorname,
        };

        let (sql_statement, sql_params) = article.build_insert();
        _ = pg_conn.execute(&sql_statement, &sql_params)?;

        let results: Vec<Article> = vec![article];

        let info = Info {
            model_name: Article::model_name(),
            action: HandlerCRUD::Create,
            extra: "".to_string(),
        };

        Ok(Response::new(Status::Successful, info, results))
    }

    fn update(req: &mut Request) -> Result<Response> {
        let pg_addr = std::env::var(DB_URL_ENV)?;
        let pg_conn = pg::Connection::open(&pg_addr)?;

        let params = req.parse_urlencoded()?;

        let id = params.get("id").ok_or(anyhow!("id error"))?.to_owned();
        let title = params
            .get("title")
            .ok_or(anyhow!("title error"))?
            .to_owned();
        let content = params
            .get("content")
            .ok_or(anyhow!("content error"))?
            .to_owned();
        let authorname = params
            .get("authorname")
            .ok_or(anyhow!("authorname error"))?
            .to_owned();

        let (sql, sql_params) = Article::build_get_by_id(id.as_str());
        let rowset = pg_conn.query(&sql, &sql_params)?;
        match rowset.rows.into_iter().next() {
            Some(row) => {
                let old_article = Article::from_row(row);

                let article = Article {
                    id,
                    title,
                    content,
                    authorname,
                    ..old_article
                };

                let (sql, sql_params) = article.build_update();
                _ = pg_conn.execute(&sql, &sql_params)?;

                let results: Vec<Article> = vec![article];

                let info = Info {
                    model_name: Article::model_name(),
                    action: HandlerCRUD::Update,
                    extra: "".to_string(),
                };

                Ok(Response::new(Status::Successful, info, results))
            }
            None => {
                bail!("update action: no item in db")
            }
        }
    }

    fn delete(req: &mut Request) -> Result<Response> {
        let pg_addr = std::env::var(DB_URL_ENV)?;
        let pg_conn = pg::Connection::open(&pg_addr)?;

        let params = req.parse_urlencoded()?;

        let id = params.get("id").ok_or(anyhow!("id error"))?.to_owned();

        let (sql, sql_params) = Article::build_delete(id.as_str());
        _ = pg_conn.execute(&sql, &sql_params)?;

        let info = Info {
            model_name: Article::model_name(),
            action: HandlerCRUD::Delete,
            extra: "".to_string(),
        };
        let results: Vec<Article> = vec![];

        Ok(Response::new(Status::Successful, info, results))
    }

    fn version(_req: &mut Request) -> Result<Response> {
        let ret = r#"{"version": 1.2}"#.to_string();
        let response = Response::from_str(Status::Successful, Default::default(), ret);

        Ok(response)
    }
}

impl Module for ArticleModule {
    fn router(&self, router: &mut Router) -> Result<()> {
        router.get("/simpletest/v1/article", Self::get_one);
        router.post("/simpletest/v1/article/new", Self::new_article);
        router.post("/simpletest/v1/article/update", Self::update);
        router.post("/simpletest/v1/article/delete", Self::delete);

        router.get("/simpletest/v1/version", Self::version);

        Ok(())
    }
}
