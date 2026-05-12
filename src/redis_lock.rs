use redis::AsyncCommands;

pub async fn try_lock_minting(
    redis: &mut redis::aio::MultiplexedConnection,  // ← promenjen tip
    vehicle_id: u64,
) -> bool {
    let key = format!("minting:vehicle:{}", vehicle_id);
    
    let result: bool = redis::cmd("SET")
        .arg(&key)
        .arg("locked")
        .arg("NX")
        .arg("EX")
        .arg(120)
        .query_async(redis)
        .await
        .unwrap_or(false);

    result
}

pub async fn release_minting_lock(
    redis: &mut redis::aio::MultiplexedConnection,  // ← promenjen tip
    vehicle_id: u64,
) {
    let key = format!("minting:vehicle:{}", vehicle_id);
    let _: () = redis.del(&key).await.unwrap_or(());
}