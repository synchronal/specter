# Play from file

This example sends H264 encoded video to a web browser.

## Usage

1. Go to [jsfiddle.net](https://jsfiddle.net/9s10amwL/).
2. Copy browser SDP offer to file `offer.txt`. 
**Important** make sure you have copied whole SDP offer, sometimes 
there might be some `=` characters at the end which might not be selected
by double-clicking.
3. Run `run.exs` from the project root directory with:

    ```
    mix run examples/play_from_file/run.exs (cat examples/play_from_file/offer.txt)
    ```

4. Copy SDP answer to the browser.
5. Click Start Session

**Important**
You have to copy SDP answer and start session quickly enough, before
the peer connection times outs (about 30s).

## Result

As a result you should see a small video (320x180) playing in your browser 
for 9 seconds.
![result](./result.png)