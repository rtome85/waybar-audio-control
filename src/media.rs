use mpris::PlaybackStatus;

pub struct MediaPlayerInfo {
    pub bus_name: String,
    pub identity: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub is_playing: bool,
}

pub fn list_players() -> Vec<MediaPlayerInfo> {
    let Ok(finder) = mpris::PlayerFinder::new() else {
        return vec![];
    };
    let Ok(players) = finder.find_all() else {
        return vec![];
    };
    players
        .into_iter()
        .filter_map(|p| {
            let status = p
                .get_playback_status()
                .unwrap_or(PlaybackStatus::Stopped);
            // Only show players that are actively playing or paused
            if matches!(status, PlaybackStatus::Stopped) {
                return None;
            }
            let identity = p.identity().to_string();
            let bus_name = p.bus_name().to_string();
            let metadata = p.get_metadata().ok();
            Some(MediaPlayerInfo {
                bus_name,
                identity,
                title: metadata
                    .as_ref()
                    .and_then(|m| m.title().map(str::to_string)),
                artist: metadata.as_ref().and_then(|m| {
                    m.artists()
                        .and_then(|a| a.into_iter().next().map(str::to_string))
                }),
                is_playing: matches!(status, PlaybackStatus::Playing),
            })
        })
        .collect()
}

fn with_player<F: FnOnce(&mpris::Player)>(bus_name: &str, f: F) {
    let Ok(finder) = mpris::PlayerFinder::new() else {
        return;
    };
    let Ok(players) = finder.find_all() else {
        return;
    };
    if let Some(p) = players.iter().find(|p| p.bus_name() == bus_name) {
        f(p);
    }
}

pub fn play_pause(bus_name: &str) {
    with_player(bus_name, |p| {
        let _ = p.play_pause();
    });
}

pub fn next_track(bus_name: &str) {
    with_player(bus_name, |p| {
        let _ = p.next();
    });
}

pub fn prev_track(bus_name: &str) {
    with_player(bus_name, |p| {
        let _ = p.previous();
    });
}
