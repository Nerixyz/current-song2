use super::*;
use crate::model::PlayInfo;

// :tf: 🤏 a tiny bit of cloning
// It's fine since we're only testing, right?

#[actix::test]
async fn basic_play() -> anyhow::Result<()> {
    let (event_tx, event_rx) = watch::channel(Arc::new(ModuleState::Paused));
    let manager = Manager::new(event_tx).start();
    let module_id = manager.send(CreateModule { priority: 1 }).await?;

    manager.send(UpdateModule::paused(module_id)).await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Paused);

    let song1 = PlayInfo::simple("Song1", "Artist1", "test");
    manager
        .send(UpdateModule::playing(module_id, song1.clone()))
        .await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Playing(song1.clone()));

    manager.send(RemoveModule { id: module_id }).await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Paused);
    manager
        .send(UpdateModule::playing(module_id, song1))
        .await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Paused);

    Ok(())
}

#[actix::test]
async fn priority_play() -> anyhow::Result<()> {
    let (event_tx, event_rx) = watch::channel(Arc::new(ModuleState::Paused));
    let manager = Manager::new(event_tx).start();
    let high_prio = manager.send(CreateModule { priority: 5 }).await?;
    let low_prio = manager.send(CreateModule { priority: 1 }).await?;

    manager.send(UpdateModule::paused(high_prio)).await?;
    manager.send(UpdateModule::paused(low_prio)).await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Paused);

    let song1 = PlayInfo::simple("Song1", "Artist1", "test");
    let song2 = PlayInfo::simple("Song2", "Artist2", "test");
    let song3 = PlayInfo::simple("Song3", "Artist3", "test");
    manager
        .send(UpdateModule::playing(low_prio, song1.clone()))
        .await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Playing(song1.clone()));
    manager
        .send(UpdateModule::playing(high_prio, song2.clone()))
        .await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Playing(song2.clone()));
    manager
        .send(UpdateModule::playing(low_prio, song3.clone()))
        .await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Playing(song2.clone()));
    manager.send(UpdateModule::paused(high_prio)).await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Playing(song3.clone()));

    manager.send(RemoveModule { id: high_prio }).await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Playing(song3.clone()));
    manager
        .send(UpdateModule::playing(high_prio, song1))
        .await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Playing(song3));
    manager.send(RemoveModule { id: low_prio }).await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Paused);
    manager.send(UpdateModule::playing(low_prio, song2)).await?;
    assert_eq!(**event_rx.borrow(), ModuleState::Paused);

    Ok(())
}
