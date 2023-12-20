<div align="Center">
<h1>avail-light</h1>
<h3> Light client for the Avail blockchain</h3>
</div>

<br>

[![Build status](https://github.com/availproject/avail-light/actions/workflows/default.yml/badge.svg)](https://github.com/availproject/avail-light/actions/workflows/default.yml) [![Code coverage](https://codecov.io/gh/availproject/avail-light/branch/main/graph/badge.svg?token=7O2EA7QMC2)](https://codecov.io/gh/availproject/avail-light)

![demo](./img/lc.png)

## Introduction

`light-client-lib` is a fork of `avail-light` and it exposes all the light client functionalities as functions that can be called either from a `C` environment or from a `JVM` environment (JVM is WIP).


## Contribution Guidelines

### Rules

Avail welcomes contributors from every background and skill level. Our mission is to build a community that's not only welcoming and friendly but also aligned with the best development practices. Interested in contributing to this project? Whether you've spotted an issue, have improvement ideas, or want to add new features, we'd love to have your input. Simply open a GitHub issue or submit a pull request to get started.

1. Before asking any questions regarding how the project works, please read through all the documentation and install the project on your own local machine to try it and understand how it works. Please ask your questions in open channels (Github and [Telegram](https://t.me/avail_uncharted/5)).

2. To work on an issue, first, get approval from a maintainer or team member. You can request to be assigned by commenting on the issue in GitHub. This respects the efforts of others who may already be working on the same issue. Unapproved PRs may be declined.

3. When assigned to an issue, it's expected that you're ready to actively work on it. After assignment, please provide a draft PR or update within one week. If you encounter delays, communicate with us to maintain your assignment.

4. Got an idea or found a bug? Open an issue with the tags [New Feature] or [Bug]. Provide detailed information like reproduction steps (for bugs) or a basic feature proposal. The team will review and potentially assign you to it.

5. Start a draft PR early in your development process, even with incomplete changes. This allows us to track progress, provide timely reviews, and assist you. Expect feedback on your drafts periodically.



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
