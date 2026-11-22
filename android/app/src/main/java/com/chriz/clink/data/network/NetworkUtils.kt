package com.chriz.clink.data.network

import android.content.Context
import android.net.ConnectivityManager
import android.net.LinkProperties
import java.net.InetAddress

object NetworkUtils {
    /**
     * Attempts to find the default gateway IP address for the currently active network.
     * When connected via USB Tethering or PC Hotspot, this gateway IP corresponds to the host PC.
     */
    fun getGatewayIp(context: Context): String? {
        try {
            val cm = context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            val network = cm?.activeNetwork
            if (network != null) {
                val linkProps = cm.getLinkProperties(network)
                if (linkProps != null) {
                    for (route in linkProps.routes) {
                        if (route.isDefaultRoute && route.gateway != null) {
                            val ip = route.gateway?.hostAddress
                            if (!ip.isNullOrBlank() && ip != "0.0.0.0" && !ip.startsWith("::")) {
                                return ip
                            }
                        }
                    }
                }
            }
        } catch (_: Exception) {
        }
        return null
    }

    /**
     * Gets the first non-loopback IPv4 address of the Android device.
     */
    fun getLocalIp(): String? {
        try {
            val interfaces = java.net.NetworkInterface.getNetworkInterfaces() ?: return null
            for (intf in interfaces) {
                val addrs = intf.inetAddresses ?: continue
                for (addr in addrs) {
                    if (!addr.isLoopbackAddress && addr is java.net.Inet4Address) {
                        val host = addr.hostAddress ?: continue
                        if (!host.startsWith("127.")) {
                            return host
                        }
                    }
                }
            }
        } catch (_: Exception) {
        }
        return null
    }
}
