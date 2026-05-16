// BermTileService.kt
// Wear OS Tile for BERM Alert.
//
// Integration: add a Wear OS module to the Android project produced by
// `expo prebuild`, register this service in its AndroidManifest, and include
// the wear-tiles + protolayout dependencies. The latest cover snapshot is
// shared from the phone via DataStore (see watch/README.md). Builds with the
// Android app via `eas build --platform android`.

package sh.berm.alert.wear

import androidx.wear.protolayout.ColorBuilders.argb
import androidx.wear.protolayout.DimensionBuilders.dp
import androidx.wear.protolayout.LayoutElementBuilders.Column
import androidx.wear.protolayout.LayoutElementBuilders.FontStyles
import androidx.wear.protolayout.LayoutElementBuilders.Layout
import androidx.wear.protolayout.LayoutElementBuilders.LayoutElement
import androidx.wear.protolayout.LayoutElementBuilders.Text
import androidx.wear.protolayout.ResourceBuilders.Resources
import androidx.wear.protolayout.TimelineBuilders.Timeline
import androidx.wear.tiles.RequestBuilders
import androidx.wear.tiles.TileBuilders.Tile
import androidx.wear.tiles.TileService
import com.google.common.util.concurrent.Futures
import com.google.common.util.concurrent.ListenableFuture

private const val RESOURCES_VERSION = "1"

private const val BREAKWATER_GREY = 0xFF2A2A2A.toInt()
private const val GLOW_CYAN = 0xFF5BC0EB.toInt()
private const val SAFETY_AMBER = 0xFFFFD93D.toInt()
private const val WARN = 0xFFFFB347.toInt()
private const val DANGER = 0xFFE5534B.toInt()

data class CoverSnapshot(
    val riskScore: Int,
    val band: String,
    val activeCovers: Int,
    val unreadAlerts: Int,
)

fun riskColor(score: Int): Int = when {
    score >= 75 -> DANGER
    score >= 50 -> WARN
    score >= 25 -> SAFETY_AMBER
    else -> GLOW_CYAN
}

class BermTileService : TileService() {

    override fun onTileRequest(
        requestParams: RequestBuilders.TileRequest
    ): ListenableFuture<Tile> {
        val snapshot = SnapshotStore.read(applicationContext)
        val tile = Tile.Builder()
            .setResourcesVersion(RESOURCES_VERSION)
            .setTileTimeline(
                Timeline.fromLayoutElement(tileLayout(snapshot))
            )
            .build()
        return Futures.immediateFuture(tile)
    }

    override fun onTileResourcesRequest(
        requestParams: RequestBuilders.ResourcesRequest
    ): ListenableFuture<Resources> {
        return Futures.immediateFuture(
            Resources.Builder().setVersion(RESOURCES_VERSION).build()
        )
    }

    private fun tileLayout(snapshot: CoverSnapshot): LayoutElement {
        return Column.Builder()
            .addContent(
                Text.Builder()
                    .setText(snapshot.riskScore.toString())
                    .setFontStyle(
                        FontStyles.display1(this)
                            .setColor(argb(riskColor(snapshot.riskScore)))
                            .build()
                    )
                    .build()
            )
            .addContent(
                Text.Builder()
                    .setText("${snapshot.band} risk")
                    .setFontStyle(
                        FontStyles.body1(this).setColor(argb(GLOW_CYAN)).build()
                    )
                    .build()
            )
            .addContent(
                Text.Builder()
                    .setText("${snapshot.activeCovers} covers / ${snapshot.unreadAlerts} alerts")
                    .setFontStyle(
                        FontStyles.caption1(this).setColor(argb(0xFFE8EAED.toInt())).build()
                    )
                    .build()
            )
            .build()
    }

    private fun Layout.padding() = dp(8f)
}
