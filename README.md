# CTC Tracker
## Purpose
This application was designed due to love of sudoku and Cracking the Cryptic. While it's easy enough to get daily puzzles from their Discord, tracking which videos I've been able to follow along with has been a more difficult challenge and one that seemed primed for a software solution.

## Authentication
This application requires a YouTube Data API key to fetch videos from the Cracking the Cryptic channel. You can get one by [registering an application with Google](https://developers.google.com/youtube/registering_an_application).

Once you have your API key, you can provide it in one of two ways:
1. **Environment Variable** (optional): Set the `CTC_API_KEY` environment variable before launching the application
2. **UI Setup Dialog**: If no API key is detected, the application will prompt you to enter it through a setup dialog on first launch

You can update your API key at any time by clicking the "⚙ Settings" button in the application.