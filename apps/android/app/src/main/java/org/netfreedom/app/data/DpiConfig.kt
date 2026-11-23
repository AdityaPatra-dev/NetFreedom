package org.netfreedom.app.data

data class DpiConfig(
    val splitPos: String = "2+s",
    val fakeTtl: Int = 3,
    val fakeSni: String = "cloudflare.com",
    val splitHttpReq: Boolean = true,
    val autoDesync: Boolean = true,
    val ttlModifier: Int = -1,
    val tcpFastOpen: Boolean = false
) {
    fun toCliArgs(listenIp: String = "127.0.0.1", listenPort: Int = 1080): List<String> {
        val args = mutableListOf<String>()
        args.add("-i")
        args.add(listenIp)
        args.add("-p")
        args.add(listenPort.toString())

        if (splitPos.isNotEmpty()) {
            args.add("-s")
            args.add(splitPos)
        }
        if (ttlModifier != 0) {
            args.add("-f")
            args.add(ttlModifier.toString())
        }
        if (fakeTtl > 0) {
            args.add("-t")
            args.add(fakeTtl.toString())
        }
        if (fakeSni.isNotEmpty()) {
            args.add("-d")
            args.add("1+s")
        }
        if (tcpFastOpen) {
            args.add("-F")
        }
        return args
    }
}
