// pub mod stacks;
// pub mod deployment_objects;

// // use super::dao::{foundations::FoundationDAO, kubernetes_objects::KubernetesObjectDAO};
// // use diesel::pg::PgConnection;
// // use diesel::Connection;
// // use std::sync::{Arc, Mutex};

// // pub struct BrokkrDAO {
// //     pub foundations: FoundationDAO,
// //     pub kubernetes_objects: KubernetesObjectDAO,
// //     conn: Arc<Mutex<PgConnection>>,
// // }

// // impl BrokkrDAO {
// //     pub fn new(conn: PgConnection) -> Self {
// //         let conn = Arc::new(Mutex::new(conn));
// //         BrokkrDAO {
// //             foundations: FoundationDAO::new(Arc::clone(&conn)),
// //             kubernetes_objects: KubernetesObjectDAO::new(Arc::clone(&conn)),
// //             conn,
// //         }
// //     }

// //     // You can add a transaction method here if needed
// //     pub fn transaction<F, T>(&self, f: F) -> Result<T, diesel::result::Error>
// //     where
// //         F: FnOnce(&Self) -> Result<T, diesel::result::Error>,
// //     {
// //         let mut conn = self.conn.lock().unwrap();
// //         conn.transaction(|_| f(self))
// //     }
// // }
