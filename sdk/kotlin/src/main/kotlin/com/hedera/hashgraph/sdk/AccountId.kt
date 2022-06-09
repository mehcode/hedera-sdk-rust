package com.hedera.hashgraph.sdk

import com.fasterxml.jackson.annotation.JsonCreator
import com.fasterxml.jackson.annotation.JsonValue
import jnr.ffi.byref.NativeLongByReference

/**
 * The unique identifier for a cryptocurrency account on Hedera.
 */
class AccountId(shard: Long, realm: Long, val num: Long) : AccountAddress(shard, realm) {
    constructor(num: Long) : this(0, 0, num)

    @JsonValue
    override fun toString(): String {
        return String.format("%d.%d.%d", shard, realm, num)
    }

    companion object {
        @JvmStatic
        @JsonCreator
        fun parse(s: String?): AccountId {
            val shard = NativeLongByReference()
            val realm = NativeLongByReference()
            val num = NativeLongByReference()

            val err = CHedera.instance.hedera_entity_id_from_string(s, shard, realm, num)

            if (err != CHedera.Error.OK) {
                throw RuntimeException("oh no")
            }

            return AccountId(shard.toLong(), realm.toLong(), num.toLong())
        }
    }
}