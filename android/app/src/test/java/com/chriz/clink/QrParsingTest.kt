package com.chriz.clink

import org.junit.Assert.*
import org.junit.Test

class QrParsingTest {

    @Test
    fun testParseClinkUri() {
        val uriStr = "clink://pair?host=192.168.1.100&port=8765&pin=654321"
        assertTrue(uriStr.startsWith("clink://"))

        val query = uriStr.substringAfter("?")
        val params = query.split("&").associate {
            val parts = it.split("=")
            parts[0] to parts.getOrElse(1) { "" }
        }

        assertEquals("192.168.1.100", params["host"])
        assertEquals("8765", params["port"])
        assertEquals("654321", params["pin"])
    }

    @Test
    fun testParseColonFormat() {
        val payload = "192.168.1.50:8765:123456"
        val parts = payload.split(":")
        assertEquals("192.168.1.50", parts[0])
        assertEquals("8765", parts[1])
        assertEquals("123456", parts[2])
    }
}
