package org.netfreedom.app.jni

import android.content.Context
import android.util.Log
import hev.htproxy.TProxyService
import java.io.File
import java.io.FileOutputStream
import java.io.IOException

object NetFreedomNative {
    private const val TAG = "NetFreedomNative"
    private var byeDpiProcess: Process? = null

    /**
     * Start the native ByeDPI daemon process.
     */
    @Synchronized
    fun startByeDpi(context: Context, args: List<String>): Boolean {
        stopByeDpi()
        try {
            val nativeDir = context.applicationInfo.nativeLibraryDir
            val binary = File(nativeDir, "libciadpi.so")
            if (!binary.exists() || !binary.canExecute()) {
                Log.e(TAG, "libciadpi.so not found or not executable at: ${binary.absolutePath}")
                return false
            }

            val command = mutableListOf<String>()
            command.add(binary.absolutePath)
            command.addAll(args)

            Log.i(TAG, "Spawning ByeDPI with cmd: ${command.joinToString(" ")}")
            val pb = ProcessBuilder(command)
            pb.redirectErrorStream(true)
            byeDpiProcess = pb.start()

            // Wait brief moment and check if process is still alive
            Thread.sleep(150)
            val isAlive = try {
                byeDpiProcess?.exitValue()
                false
            } catch (e: IllegalThreadStateException) {
                true
            }

            Log.i(TAG, "ByeDPI process started successfully. Running: $isAlive")
            return isAlive
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start ByeDPI process", e)
            return false
        }
    }

    /**
     * Stop the native ByeDPI daemon.
     */
    @Synchronized
    fun stopByeDpi(): Boolean {
        byeDpiProcess?.let { p ->
            try {
                p.destroy()
                p.waitFor()
            } catch (ignored: Exception) {
            }
        }
        byeDpiProcess = null
        return true
    }

    fun isByeDpiRunning(): Boolean {
        return try {
            byeDpiProcess?.exitValue()
            false
        } catch (e: IllegalThreadStateException) {
            true
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Generate hev-socks5-tunnel YAML configuration.
     */
    fun createHevConfig(
        cacheDir: File,
        socksHost: String = "127.0.0.1",
        socksPort: Int = 1080,
        mtu: Int = 1420
    ): File {
        val configFile = File(cacheDir, "hev-tunnel.yml")
        val configContent = """
            tunnel:
              mtu: $mtu
            
            socks5:
              port: $socksPort
              address: '$socksHost'
              udp: 'tcp'
            
            misc:
              limit-nofile: 4096
        """.trimIndent()

        FileOutputStream(configFile).use { out ->
            out.write(configContent.toByteArray())
        }
        return configFile
    }

    /**
     * Start the hev-socks5-tunnel against the established TUN file descriptor.
     */
    fun startHevTunnel(configFile: File, tunFd: Int): Boolean {
        return try {
            TProxyService.TProxyStartService(configFile.absolutePath, tunFd)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start hev-socks5-tunnel", e)
            false
        }
    }

    /**
     * Stop the hev-socks5-tunnel.
     */
    fun stopHevTunnel(): Boolean {
        return try {
            TProxyService.TProxyStopService()
        } catch (e: Exception) {
            Log.e(TAG, "Failed to stop hev-socks5-tunnel", e)
            false
        }
    }

    /**
     * Returns array: [tx_packets, tx_bytes, rx_packets, rx_bytes]
     */
    fun getTunnelStats(): LongArray {
        return try {
            TProxyService.TProxyGetStats() ?: LongArray(4)
        } catch (e: Exception) {
            LongArray(4)
        }
    }
}
