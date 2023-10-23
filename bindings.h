#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define CELL_WITH_PROOF_SIZE (CELL_SIZE + PROOF_SIZE)

typedef void (*FfiCallback)(const uint8_t *data);

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
