const songInfo = document.getElementById('song-info');
// create a new h3
const albumEl = document.createElement('h3');
// this id will be used by wrap() too and can be used for styling
albumEl.id = 'album';
songInfo.append(albumEl);
// add the marquee effect to our element
// the returned object can be used to control the effect
const albumMarquee = cso2.marquee.wrap(albumEl);

export function onPlay(state) {
    if (!state.info.album?.title) {
        // no album with a title, hide it
        albumEl.classList.add('hidden');
        return;
    }
    albumEl.classList.remove('hidden');

    albumMarquee.start(); // start the animation (no-op if it's already started)
    if (albumEl.textContent != state.info.album.title) {
        albumMarquee.reset(); // start from the beginning
        albumEl.textContent = state.info.album.title; // set the visible text
    }
}

export function onPause() {
    // pause the animation (unregisters a requestAnimationFrame handler)
    albumMarquee.pause();
}
