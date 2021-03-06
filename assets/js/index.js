import '../css/reset.css'
import 'wingcss'
import '../css/index.css'
import '../css/login.css'
import '../css/pages/profile.css'
import '../css/pages/user-comments-page.css'
import '../css/pages/tags-page.css'
import '../css/pages/logs-page.css'

import React from 'react'
import ReactDOM from 'react-dom'
import Plyr from 'plyr'

import UploadPage from './pages/upload'

import SearchBox from './components/search_box/form'
import SearchBoxInput from './components/search_box'
import TwitterUploader from './components/twitter_uploader'

import './lib/upload_tooltips'

if (document.getElementById('search-box-form')) {
  let container = document.getElementById('search-box-form')
  let query = container.dataset.query || ''

  if (!query.length) {
    query = ''
  }

  let props = {
    query,
  }

  ReactDOM.render(<SearchBox {...props} />, container)
}

if (document.getElementById('tag-box-input')) {
  let container = document.getElementById('tag-box-input')
  let query = container.dataset.tags || ''
  let name = container.dataset.name || ''

  if (!query.length) {
    query = ''
  }

  let props = {
    query,
    name,
    required: true,
  }

  ReactDOM.render(<SearchBoxInput {...props} />, container)
}

if (document.getElementById('upload-page')) {
  let page = document.getElementById('upload-page')

  ReactDOM.render(<UploadPage />, page)
}

if (document.getElementById('twitter-uploader')) {
  let $el = document.getElementById('twitter-uploader')
  ReactDOM.render(<TwitterUploader />, $el)
}

window.addEventListener('DOMContentLoaded', () => {
  const frameTime = 1 / 30
  let $video = document.getElementById('video-player')

  if ($video) {
    const player = new Plyr('#video-player', {
      speed: {
        selected: 1,
        options: [0.1, 0.25, 0.5, 0.75, 1],
      },
      keyboard: {
        focused: true,
        global: true,
      },
      controls: [
        'restart', // Restart playback
        'play', // Play/pause playback
        'progress', // The progress bar and scrubber for playback and buffering
        'current-time', // The current time of playback
        'duration', // The full duration of the media
        'mute', // Toggle mute
        'volume', // Volume control
        'captions', // Toggle captions
        'settings', // Settings menu
        'pip', // Picture-in-picture (currently Safari only)
        'airplay', // Airplay (currently Safari only)
        'fullscreen', // Toggle fullscreen
      ],
      enabled: !/webOS|iPhone|iPod|BlackBerry/i.test(navigator.userAgent),
    })

    window.addEventListener('keypress', function (evt) {
      if (evt.keyCode === 122) {
        player.elements.container.classList.toggle('flipped')
      }

      if (player.paused) {
        if (evt.keyCode === 44) {
          // ',' = back one frame
          player.currentTime = Math.max(0, player.currentTime - frameTime)
        } else if (evt.keyCode === 46) {
          // '.' = forward one frame
          player.currentTime = Math.min(
            player.duration,
            player.currentTime + frameTime
          )
        }
      }
    })
  }

  let $markdownContainer = document.querySelector('textarea#content')

  if ($markdownContainer) {
    let easyMDE = new EasyMDE({
      element: $markdownContainer,
      spellChecker: false,
    })
  }
})
