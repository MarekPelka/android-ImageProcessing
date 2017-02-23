/*
 * Copyright (C) 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package com.example.android.imageProcessing;

import android.graphics.ImageFormat;
import android.os.Handler;
import android.os.HandlerThread;
import android.renderscript.Allocation;
import android.renderscript.Element;
import android.renderscript.RenderScript;
import android.renderscript.Type;
import android.util.Size;
import android.view.Surface;

/**
 * Renderscript-based merger for an HDR viewfinder
 */
public class ViewfinderProcessor {

    private Allocation mInputHdrAllocation;
    private Allocation mInputNormalAllocation;


    private Allocation mPrevGrayAllocation;
    private Allocation mPrevBlurrAllocation;
    private Allocation mPrevGXAllocation;
    private Allocation mPrevGYAllocation;
    private Allocation mPrevEdgeAllocation;
    private Allocation mPrevCornerAllocation;

    private Allocation mTrashFloatAllocation;
    private Allocation mTrashCharAllocation;

    private Allocation mPrevAllocation;
    private Allocation mOutputAllocation;

    private Surface mOutputSurface;
    private HandlerThread mProcessingThread;
    private Handler mProcessingHandler;
    private ScriptC_corner_processing mHdrMergeScript;

    public ProcessingTask mNormalTask;
    public ProcessingTask mGrayTask;
    public ProcessingTask mGXTask;
    public ProcessingTask mGYTask;
    public ProcessingTask mEdgeTask;
    public ProcessingTask mCornerTask;

    private Size mSize;

    private int mMode;

    public final static int MODE_NORMAL = 0;
    public final static int MODE_GRAY = 1;
    public final static int MODE_BLUR = 2;
    public final static int MODE_G_X = 3;
    public final static int MODE_G_Y = 4;
    public final static int MODE_EDGE = 5;
    public final static int MODE_CORNER = 6;

    public ViewfinderProcessor(RenderScript rs, Size dimensions) {

        mSize = dimensions;

        Type.Builder yuvTypeBuilder = new Type.Builder(rs, Element.YUV(rs));
        yuvTypeBuilder.setX(dimensions.getWidth());
        yuvTypeBuilder.setY(dimensions.getHeight());
        yuvTypeBuilder.setYuvFormat(ImageFormat.YUV_420_888);
        mInputHdrAllocation = Allocation.createTyped(rs, yuvTypeBuilder.create(),
                Allocation.USAGE_IO_INPUT | Allocation.USAGE_SCRIPT);
        mInputNormalAllocation = Allocation.createTyped(rs, yuvTypeBuilder.create(),
                Allocation.USAGE_IO_INPUT | Allocation.USAGE_SCRIPT);

        Type.Builder floatBuilder = new Type.Builder(rs, Element.F32(rs));
        floatBuilder.setX(dimensions.getWidth());
        floatBuilder.setY(dimensions.getHeight());

        mPrevBlurrAllocation = Allocation.createTyped(rs, floatBuilder.create(), Allocation.USAGE_SCRIPT);
        mPrevGXAllocation = Allocation.createTyped(rs, floatBuilder.create(), Allocation.USAGE_SCRIPT);
        mPrevGYAllocation = Allocation.createTyped(rs, floatBuilder.create(), Allocation.USAGE_SCRIPT);
        mPrevCornerAllocation = Allocation.createTyped(rs, floatBuilder.create(), Allocation.USAGE_SCRIPT);
        mTrashFloatAllocation = Allocation.createTyped(rs, floatBuilder.create(), Allocation.USAGE_SCRIPT);

        Type.Builder rgbTypeBuilder = new Type.Builder(rs, Element.RGBA_8888(rs));
        rgbTypeBuilder.setX(dimensions.getWidth());
        rgbTypeBuilder.setY(dimensions.getHeight());
        mTrashCharAllocation = Allocation.createTyped(rs, rgbTypeBuilder.create(), Allocation.USAGE_SCRIPT);
        mPrevGrayAllocation = Allocation.createTyped(rs, rgbTypeBuilder.create(), Allocation.USAGE_SCRIPT);
        mOutputAllocation = Allocation.createTyped(rs, rgbTypeBuilder.create(),
                Allocation.USAGE_IO_OUTPUT | Allocation.USAGE_SCRIPT);

        mProcessingThread = new HandlerThread("ViewfinderProcessor");
        mProcessingThread.start();
        mProcessingHandler = new Handler(mProcessingThread.getLooper());

        mHdrMergeScript = new ScriptC_corner_processing(rs);

        //mHdrMergeScript.set_gCurrentBlurredFrame(mPrevAllocation);

        //mEdgeTask = new ProcessingTask(mInputHdrAllocation, dimensions.getWidth(), dimensions.getHeight(), mMode);
        mNormalTask = new ProcessingTask(mInputNormalAllocation, dimensions.getWidth(), dimensions.getHeight(), mMode);

        setRenderMode(MODE_NORMAL);
    }

    public Surface getInputHdrSurface() {
        return mInputHdrAllocation.getSurface();
    }

    public Surface getInputNormalSurface() {
        return mInputNormalAllocation.getSurface();
    }

    public void setOutputSurface(Surface output) {
        mOutputAllocation.setSurface(output);
    }

    public void setRenderMode(int mode) {
        mMode = mode;
    }

    /**
     * Simple class to keep track of incoming frame count,
     * and to process the newest one in the processing thread
     */
    class ProcessingTask implements Runnable, Allocation.OnBufferAvailableListener {
        private int mPendingFrames = 0;
        private int mFrameCounter = 0;
        private int mWidth;
        private int mHeight;

        private Allocation mInputAllocation;

        public ProcessingTask(Allocation input, int width, int height, int mode) {
            mInputAllocation = input;
            mInputAllocation.setOnBufferAvailableListener(this);
            mWidth = width;
            mHeight = height;
        }

        @Override
        public void onBufferAvailable(Allocation a) {
            synchronized(this) {
                mPendingFrames++;
                mProcessingHandler.post(this);
            }
        }

        @Override
        public void run() {

            // Find out how many frames have arrived
            int pendingFrames;
            synchronized(this) {
                pendingFrames = mPendingFrames;
                mPendingFrames = 0;

                // Discard extra messages in case processing is slower than frame rate
                mProcessingHandler.removeCallbacks(this);
            }

            // Get to newest input
            for (int i = 0; i < pendingFrames; i++) {
                mInputAllocation.ioReceive();
            }

            mHdrMergeScript.set_gFrameCounter(mFrameCounter++);
            mHdrMergeScript.set_gCurrentFrame(mInputAllocation);
            mHdrMergeScript.set_gGrayFrame(mPrevGrayAllocation);

            mHdrMergeScript.set_gBlurredFrame(mPrevBlurrAllocation);
            mHdrMergeScript.set_gAllGradientHorizontal(mPrevGXAllocation);
            mHdrMergeScript.set_gAllGradientVertical(mPrevGYAllocation);
            mHdrMergeScript.set_gComCorners(mPrevCornerAllocation);

            mHdrMergeScript.set_gWidth(mWidth);
            mHdrMergeScript.set_gHeight(mHeight);
            mHdrMergeScript.set_gMode(mMode);

            // Run processing pass
            if (mMode == MODE_NORMAL || mMode == MODE_GRAY)
                mHdrMergeScript.forEach_gray(mOutputAllocation);
            else if (mMode == MODE_BLUR) {

                mHdrMergeScript.forEach_gray(mTrashCharAllocation);
                mHdrMergeScript.forEach_blur(mTrashFloatAllocation);
                mHdrMergeScript.forEach_blur_to_img(mOutputAllocation);
//                mHdrMergeScript.forEach_gY(mPrevGYAllocation);
//                mHdrMergeScript.forEach_edge(mInputAllocation, mOutputAllocation);
            } else if (mMode == MODE_G_X) {
                mHdrMergeScript.forEach_gray(mTrashCharAllocation);
                mHdrMergeScript.forEach_blur(mTrashFloatAllocation);
                mHdrMergeScript.forEach_gX(mOutputAllocation);
            } else if (mMode == MODE_G_Y) {
                mHdrMergeScript.forEach_gray(mTrashCharAllocation);
	            mHdrMergeScript.forEach_blur(mTrashFloatAllocation);
                mHdrMergeScript.forEach_gY(mOutputAllocation);
            } else if (mMode == MODE_EDGE) {
                mHdrMergeScript.forEach_gray(mTrashCharAllocation);
                mHdrMergeScript.forEach_blur(mTrashFloatAllocation);
                mHdrMergeScript.forEach_gX(mTrashCharAllocation);
                mHdrMergeScript.forEach_gY(mTrashCharAllocation);
                mHdrMergeScript.forEach_edge(mOutputAllocation);
            } else if (mMode == MODE_CORNER) {
                mHdrMergeScript.forEach_gray(mTrashCharAllocation);
                mHdrMergeScript.forEach_blur(mTrashFloatAllocation);
                mHdrMergeScript.forEach_gX(mTrashCharAllocation);
                mHdrMergeScript.forEach_gY(mTrashCharAllocation);
                mHdrMergeScript.forEach_harris(mTrashCharAllocation);
                //mHdrMergeScript.forEach_nonMaxSuppression(mTrashFloatAllocation, mTrashFloatAllocation);
                mHdrMergeScript.forEach_draw(mOutputAllocation);
            }
            mOutputAllocation.ioSend();
        }
    }

}
