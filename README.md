# Android-ImageProcessing
This app shows effects of real time image procesing on Android using Render Script. 

Introduction
--------------
Originally this program supposed to be Harris corner detector but its still wokr in progress. This application 
was based on Google's [HdrViewfinder][1] sample. All license information from originals files were kept. This demo 
was created using information from [reVision-renderscript][2] which implements Harris corner detection and [OnionCamera][3] 
that implements Canny edge detection in real time.

[1]: https://github.com/googlesamples/android-HdrViewfinder
[2]: https://github.com/v4vision/reVision-renderscript
[3]: https://github.com/arekolek/OnionCamera

Technologies
--------------

- RenderScript
- Java

To do
--------------
1. Fix corner suppression in order to show only strongest corners on edge.
2. Upgrade Sobel edge algorithm to Canny's edge detection.
3. Organize memory allocation for RenderScript.