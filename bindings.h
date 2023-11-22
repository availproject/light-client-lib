#include <stdarg.h>
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
  (JNIEnv *, jclass, jstring, jint, jbool, jbool);