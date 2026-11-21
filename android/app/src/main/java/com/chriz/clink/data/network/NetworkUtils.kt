package com.chriz.clink.data.network

import android.content.Context
import android.net.ConnectivityManager
import android.net.LinkProperties
import java.net.InetAddress

object NetworkUtils {
    /**
     * Attempts to find the default gateway IP address for the currently active network.
     * When connected via USB Tethering, this gateway IP corresponds to the host Linux PC.
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
}
