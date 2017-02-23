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
#pragma version(1)
#pragma rs java_package_name(com.example.android.imageProcessing)
#pragma rs_fp_relaxed

rs_allocation gCurrentFrame;
rs_allocation gGrayFrame;
rs_allocation gBlurredFrame;
rs_allocation gAllGradientHorizontal;
rs_allocation gAllGradientVertical;
rs_allocation gComCorners;

int gMode = 0;
int gSuppressRadius = 3;

int gHeight = 0;
int gWidth = 0;

float c = 0.04;
float harrisThreshold = 300000000;

int gFrameCounter = 0;

uchar4 __attribute__((kernel)) gray(uint32_t x, uint32_t y) {

    int4 outPixel;
    outPixel.a = 255;

    if(gMode == 0) {
        uchar4 curPixel;
        curPixel.r = rsGetElementAtYuv_uchar_Y(gCurrentFrame, x, y);
        curPixel.g = rsGetElementAtYuv_uchar_U(gCurrentFrame, x, y);
        curPixel.b = rsGetElementAtYuv_uchar_V(gCurrentFrame, x, y);
        curPixel.a = 255;

        outPixel.r = curPixel.r +
                    curPixel.b * 1436 / 1024 - 179;
        outPixel.g = curPixel.r -
                    curPixel.g * 46549 / 131072 + 44 -
                    curPixel.b * 93604 / 131072 + 91;
        outPixel.b = curPixel.r +
                    curPixel.g * 1814 / 1024 - 227;
    } else {
        outPixel.r = rsGetElementAtYuv_uchar_Y(gCurrentFrame, x, y);
        outPixel.g = outPixel.r;
        outPixel.b = outPixel.r;
    }

    uchar4 out = convert_uchar4(clamp(outPixel, 0, 255));
    rsSetElementAt_uchar4(gGrayFrame, out, x, y);
    return out;

}

inline static float getElementAtFloat(rs_allocation in, uint32_t x,
		uint32_t y) {
	return rsGetElementAtYuv_uchar_Y(in, x, y);
}

float __attribute__((kernel)) blur(uint32_t x, uint32_t y) {

    float pixel = 0;

    if(x == 0 || y == 0 || x == gWidth || y == gHeight) {
        return pixel;
    } else if (x == 1 || y == 1 || x == gWidth - 1 || y == gHeight - 1) {

        pixel += 9 * getElementAtFloat(gCurrentFrame, x - 1, y - 1);
        pixel += 12 * getElementAtFloat(gCurrentFrame, x, y - 1);
        pixel += 9 * getElementAtFloat(gCurrentFrame, x + 1, y - 1);

        pixel += 12 * getElementAtFloat(gCurrentFrame, x - 1, y);
        pixel += 15 * getElementAtFloat(gCurrentFrame, x, y);
        pixel += 12 * getElementAtFloat(gCurrentFrame, x + 1, y);

        pixel += 9 * getElementAtFloat(gCurrentFrame, x - 1, y + 1);
        pixel += 12 * getElementAtFloat(gCurrentFrame, x, y + 1);
        pixel += 9 * getElementAtFloat(gCurrentFrame, x + 1, y + 1);

        pixel /= 101;
        rsSetElementAt_float(gBlurredFrame, pixel, x, y);
        return pixel;
    } else {

        pixel += 2 * getElementAtFloat(gCurrentFrame, x - 2, y - 2);
        pixel += 4 * getElementAtFloat(gCurrentFrame, x - 1, y - 2);
        pixel += 5 * getElementAtFloat(gCurrentFrame, x,     y - 2);
        pixel += 4 * getElementAtFloat(gCurrentFrame, x + 1, y - 2);
        pixel += 2 * getElementAtFloat(gCurrentFrame, x + 2, y - 2);

        pixel += 4 * getElementAtFloat(gCurrentFrame, x - 2, y - 1);
        pixel += 9 * getElementAtFloat(gCurrentFrame, x - 1, y - 1);
        pixel += 12 * getElementAtFloat(gCurrentFrame, x,    y - 1);
        pixel += 9 * getElementAtFloat(gCurrentFrame, x + 1, y - 1);
        pixel += 4 * getElementAtFloat(gCurrentFrame, x + 2, y - 1);

        pixel += 5 * getElementAtFloat(gCurrentFrame, x - 2,  y);
        pixel += 12 * getElementAtFloat(gCurrentFrame, x - 1, y);
        pixel += 15 * getElementAtFloat(gCurrentFrame, x    , y);
        pixel += 12 * getElementAtFloat(gCurrentFrame, x + 1, y);
        pixel += 5 * getElementAtFloat(gCurrentFrame, x + 2,  y);

        pixel += 4 * getElementAtFloat(gCurrentFrame, x - 2, y + 1);
        pixel += 9 * getElementAtFloat(gCurrentFrame, x - 1, y + 1);
        pixel += 12 * getElementAtFloat(gCurrentFrame, x,    y + 1);
        pixel += 9 * getElementAtFloat(gCurrentFrame, x + 1, y + 1);
        pixel += 4 * getElementAtFloat(gCurrentFrame, x + 2, y + 1);

        pixel += 2 * getElementAtFloat(gCurrentFrame, x - 2, y + 2);
        pixel += 4 * getElementAtFloat(gCurrentFrame, x - 1, y + 2);
        pixel += 5 * getElementAtFloat(gCurrentFrame, x,     y + 2);
        pixel += 4 * getElementAtFloat(gCurrentFrame, x + 1, y + 2);
        pixel += 2 * getElementAtFloat(gCurrentFrame, x + 2, y + 2);

        pixel /= 159;
        rsSetElementAt_float(gBlurredFrame, pixel, x, y);
        return pixel;
    }
}

uchar4 __attribute__((kernel)) blur_to_img(uint32_t x, uint32_t y) {

    float blurP = 0;

    if(x == 0 || y == 0 || x == gWidth || y == gHeight)
    {
        //Do nothing
    } else {

        blurP = rsGetElementAt_float(gBlurredFrame, x, y);
    }
    // Stors gradient result
    //rsSetElementAt_float(gAllGradientHorizontal, blurP, x, y);

    if(gMode == 2) {
        int4 outPixel;
        outPixel.a = 255;
        blurP = blurP / 8.0f + 128;
        if(blurP < 0){
            outPixel.r = 0;
            outPixel.g = 0;
            outPixel.b = 0;
        } else if (blurP > 255) {
            outPixel.r = 255;
            outPixel.g = 255;
            outPixel.b = 255;
        } else {
            outPixel.r = blurP;
            outPixel.g = blurP;
            outPixel.b = blurP;
        }

        uchar4 out = convert_uchar4(clamp(outPixel, 0, 255));
        return out;
    } else {
        return blurP;
    }
}

uchar4 __attribute__((kernel)) gX(uint32_t x, uint32_t y) {

    float gx = 0;

    if(x == 0 || y == 0 || x == gWidth || y == gHeight)
    {
        //Do nothing
    } else {

        gx -= rsGetElementAt_float(gBlurredFrame, x - 1, y - 1);
        gx -= rsGetElementAt_float(gBlurredFrame, x - 1, y) * 2;
        gx -= rsGetElementAt_float(gBlurredFrame, x - 1, y + 1);
        gx += rsGetElementAt_float(gBlurredFrame, x + 1, y - 1);
        gx += rsGetElementAt_float(gBlurredFrame, x + 1, y) * 2;
        gx += rsGetElementAt_float(gBlurredFrame, x + 1, y + 1);
    }
    // Stors gradient result
    rsSetElementAt_float(gAllGradientHorizontal, gx, x, y);

        int4 outPixel;
        outPixel.a = 255;

    if(gMode == 3) {

        gx = gx / 8.0f + 128;
        if(gx < 0){
            outPixel.r = 0;
            outPixel.g = 0;
            outPixel.b = 0;
        } else if (gx > 255) {
            outPixel.r = 255;
            outPixel.g = 255;
            outPixel.b = 255;
        } else {
            outPixel.r = gx;
            outPixel.g = gx;
            outPixel.b = gx;
        }

        uchar4 out = convert_uchar4(clamp(outPixel, 0, 255));
        return out;
    } else {
        outPixel.r = gx;
        outPixel.g = gx;
        outPixel.b = gx;
        uchar4 out = convert_uchar4(clamp(outPixel, 0, 255));
        return out;
    }
}

uchar4 __attribute__((kernel)) gY(uint32_t x, uint32_t y) {

    float gy = 0;

    if(x == 0 || y == 0 || x == gWidth || y == gHeight)
    {
        //Do nothing
    } else {

        gy -= rsGetElementAt_float(gBlurredFrame, x - 1, y - 1);
        gy -= rsGetElementAt_float(gBlurredFrame, x, y - 1) * 2;
        gy -= rsGetElementAt_float(gBlurredFrame, x + 1, y - 1);
        gy += rsGetElementAt_float(gBlurredFrame, x - 1, y + 1);
        gy += rsGetElementAt_float(gBlurredFrame, x, y + 1) * 2;
        gy += rsGetElementAt_float(gBlurredFrame, x + 1, y + 1);
    }
    // Stors gradient result
    rsSetElementAt_float(gAllGradientVertical, gy, x, y);

        int4 outPixel;
        outPixel.a = 255;

    if(gMode == 4) {

        gy = gy / 8.0f + 128;
        if(gy < 0){
            outPixel.r = 0;
            outPixel.g = 0;
            outPixel.b = 0;
        } else if (gy > 255) {
            outPixel.r = 255;
            outPixel.g = 255;
            outPixel.b = 255;
        } else {
            outPixel.r = gy;
            outPixel.g = gy;
            outPixel.b = gy;
        }

        uchar4 out = convert_uchar4(clamp(outPixel, 0, 255));
        return out;
    } else {
        outPixel.r = gy;
        outPixel.g = gy;
        outPixel.b = gy;
        uchar4 out = convert_uchar4(clamp(outPixel, 0, 255));
        return out;
    }
}

uchar4 __attribute__((kernel)) edge(uint32_t x, uint32_t y) {

    int4 mergedPixel;
    mergedPixel.a = 255;
    if(x == 0 || y == 0 || x == gWidth || y == gHeight)
    {
        mergedPixel.r = 0;
        mergedPixel.g = 0;
        mergedPixel.b = 0;
    } else {
        float gx = rsGetElementAt_float(gAllGradientHorizontal, x, y);

        float gy = rsGetElementAt_float(gAllGradientVertical, x, y);

        int edge = ((int) round(sqrt(gx*gx + gy*gy) / 1443.0f * 255.0f));

        if(gx < 0){
            mergedPixel.r = 0;
            mergedPixel.g = 0;
            mergedPixel.b = 0;
        } else if (gx > 255) {
            mergedPixel.r = 255;
            mergedPixel.g = 255;
            mergedPixel.b = 255;
        } else {
            mergedPixel.r = edge;
            mergedPixel.g = edge;
            mergedPixel.b = edge;
        }
    }

    uchar4 out = convert_uchar4(clamp(mergedPixel, 0, 255));

    return out;
}

uchar4 __attribute__((kernel)) harris(uint32_t x, uint32_t y)
{
    float Ix = rsGetElementAt_float(gAllGradientHorizontal, x, y);
    float Iy = rsGetElementAt_float(gAllGradientVertical, x, y);

    float Ixx = Ix * Ix;
    float Iyy = Iy * Iy;
    float Ixy = Ix * Iy;

    float cornerResponse = (Ixx * Iyy - Ixy * Ixy - c * (Ixx + Iyy) * (Ixx + Iyy));

    uchar4 outPixel;
    uchar4 out;
    outPixel.r = rsGetElementAtYuv_uchar_Y(gCurrentFrame, x, y);
    outPixel.g = rsGetElementAtYuv_uchar_U(gCurrentFrame, x, y);
    outPixel.b = rsGetElementAtYuv_uchar_V(gCurrentFrame, x, y);
    outPixel.a = 255;

    if(cornerResponse < -harrisThreshold || cornerResponse > harrisThreshold) {
        out.r = 255;
        out.g = 0;
        out.b = 0;
        rsSetElementAt_float(gComCorners, cornerResponse, x, y);
    } else {

        out.r = outPixel.r +
                    outPixel.b * 1436 / 1024 - 179;
        out.g = outPixel.r -
                    outPixel.g * 46549 / 131072 + 44 -
                    outPixel.b * 93604 / 131072 + 91;
        out.b = outPixel.r +
                    outPixel.g * 1814 / 1024 - 227;
        rsSetElementAt_float(gComCorners, 0, x, y);
    }

    return out;
}

float __attribute__((kernel)) nonMaxSuppression(float in, uint32_t x, uint32_t y)
{
if(x < gSuppressRadius + 1 || y < gSuppressRadius + 1 || x > gWidth - gSuppressRadius - 1|| y > gHeight - gSuppressRadius - 1)
return in;
    float thisResponse = rsGetElementAt_float(gComCorners, x, y);
    float comparator = 0;
    for(int ky = -1 * gSuppressRadius; (thisResponse != 0 && ky <= gSuppressRadius); ky++) {
        for(int kx = -1 * gSuppressRadius; kx <= gSuppressRadius; kx++) {
        int a = x + kx;
        int b = y + ky;
            //if(a < 0 || b < 0 || a > gWidth || b > gHeight){
                //comparator = 0;
            //} else {
                comparator = rsGetElementAt_float(gComCorners, a, b);
            //}

            if(comparator < -thisResponse || comparator > thisResponse) {
                thisResponse = 0;
                break;
            }
        }
    }
    rsSetElementAt_float(gComCorners, thisResponse, x, y);
    return in;
}

uchar4 __attribute__((kernel)) draw(uint32_t x, uint32_t y)
{
       int4 out;
   if(rsGetElementAt_float(gComCorners, x, y) != 0) {
       out.r = 255;
       out.g = 0;
       out.b = 0;
   } else {
       uchar4 outPixel;

       outPixel.r = rsGetElementAtYuv_uchar_Y(gCurrentFrame, x, y);
       outPixel.g = rsGetElementAtYuv_uchar_U(gCurrentFrame, x, y);
       outPixel.b = rsGetElementAtYuv_uchar_V(gCurrentFrame, x, y);
       outPixel.a = 255;

       out.r = outPixel.r +
                   outPixel.b * 1436 / 1024 - 179;
       out.g = outPixel.r -
                   outPixel.g * 46549 / 131072 + 44 -
                   outPixel.b * 93604 / 131072 + 91;
       out.b = outPixel.r +
                   outPixel.g * 1814 / 1024 - 227;

   }
   uchar4 o = convert_uchar4(clamp(out, 0, 255));
   return o;
}