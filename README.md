<div align="Center">
<h1>avail-light</h1>
<h3> Light client for the Avail blockchain</h3>
</div>

<br>

[![Build status](https://github.com/availproject/avail-light/actions/workflows/default.yml/badge.svg)](https://github.com/availproject/avail-light/actions/workflows/default.yml) [![Code coverage](https://codecov.io/gh/availproject/avail-light/branch/main/graph/badge.svg?token=7O2EA7QMC2)](https://codecov.io/gh/availproject/avail-light)

![demo](./img/lc.png)

## Introduction

`light-client-lib` is a fork of `avail-light` and it exposes all the light client functionalities as functions that can be called either from a `C` environment or from a `JVM` environment (JVM is WIP).

## Setup

To compile this light client for Android please follow these steps given in this [link](https://avail-project.notion.site/avail-project/Compiling-Light-client-for-Android-c5db97cf21554c0bb7536d23c35174f8) carefully.

## steps to call pre-compiled lib from android app.

1. Create dir in your project at src/main/jniLibs/arm64-v8a
2. Place compiled libavail_light.so in this directory.
3. Download and place this [file](https://github.com/availproject/avail-lc-android-lib/blob/main/src/main/jniLibs/arm64-v8a/libc%2B%2B_shared.so) in the same directoy.
4. Create libavail_light.h with following content.

````#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <jni.h>
#include <android/log.h>

#define CELL_WITH_PROOF_SIZE (CELL_SIZE + PROOF_SIZE)

typedef void (*FfiCallback)(const uint8_t *data);


__android_log_write(ANDROID_LOG_ERROR, "Tag", "Error here");

bool startLightNode(uint8_t *cfg);

const uint8_t *latestBlock(uint8_t *cfg);

const uint8_t *status(uint32_t app_id, uint8_t *cfg);

const uint8_t *confidence(uint32_t block, uint8_t *cfg);

bool startLightNodeWithCallback(uint8_t *cfg, const FfiCallback *ffi_callback);

const uint8_t *submitTransactionn(uint8_t *cfg,
                                  uint32_t app_id,
                                  uint8_t *transaction,
                                  uint8_t *private_key);

const uint8_t *getStatusV2(uint8_t *cfg);

JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_startNode
  (JNIEnv *, jclass, jstring);

JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_startNodeWithBroadcastsToDb
  (JNIEnv *, jclass, jstring);

JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_latestBlock
  (JNIEnv *, jclass, jstring);
JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_confidence
  (JNIEnv *, jclass, jstring, jint);
JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_status
  (JNIEnv *, jclass, jstring, jint);
JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_getStatusV2
  (JNIEnv *, jclass, jstring);


JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_startNodeWithCallback
  (JNIEnv *, jclass, jstring, jobject);


JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_getConfidenceMessageList
  (JNIEnv *, jclass, jstring);
JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_getHeaderVerifiedMessageList
  (JNIEnv *, jclass, jstring);
JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_getDataVerifiedMessageList
  (JNIEnv *, jclass, jstring);


JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_getBlock
  (JNIEnv *, jclass, jstring);

JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_getBlockHeader
  (JNIEnv *, jclass, jstring, jint);

JNIEXPORT jstring JNICALL Java_com_example_availlibrary_AvailLightClientLib_getBlockData
  (JNIEnv *, jclass, jstring, jint, jbool, jbool);```
````

5. Now you can call any of the functions directly in the app, like to call Get block data can be using getBlockData(string, jint, bool, bool);
6. Make sure for calling to from java function name needs to be lead by application package ID, so for different package ids [here](https://github.com/availproject/light-client-lib/blob/feat/android/api-v2/src/api/v1/ffi_api/jni_ffi.rs) and [here](https://github.com/availproject/light-client-lib/blob/feat/android/api-v2/src/api/v2/ffi_api/jni_ffi.rs).
7. You can use this sample kotlin library as an [example](https://github.com/availproject/light-client-lib/tree/feat/android/api-v2). Make sure to replace sym-links in src/main/jniLibs/arm64-v8a with your compiled libs.
