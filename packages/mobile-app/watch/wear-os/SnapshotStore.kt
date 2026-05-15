// SnapshotStore.kt
// Reads the latest cover snapshot shared from the phone app.
//
// The phone writes a JSON snapshot into SharedPreferences ("berm_wear") which is
// synchronised to the watch via the Wearable Data Layer DataClient. The Tile
// service reads the most recent value here.

package sh.berm.alert.wear

import android.content.Context
import org.json.JSONObject

object SnapshotStore {
    private const val PREFS = "berm_wear"
    private const val KEY = "coverSnapshot"

    fun read(context: Context): CoverSnapshot {
        val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        val raw = prefs.getString(KEY, null) ?: return empty()
        return runCatching {
            val json = JSONObject(raw)
            CoverSnapshot(
                riskScore = json.optInt("riskScore", 0),
                band = json.optString("band", "CALM"),
                activeCovers = json.optInt("activeCovers", 0),
                unreadAlerts = json.optInt("unreadAlerts", 0),
            )
        }.getOrDefault(empty())
    }

    fun write(context: Context, snapshot: CoverSnapshot) {
        val json = JSONObject()
            .put("riskScore", snapshot.riskScore)
            .put("band", snapshot.band)
            .put("activeCovers", snapshot.activeCovers)
            .put("unreadAlerts", snapshot.unreadAlerts)
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
            .edit()
            .putString(KEY, json.toString())
            .apply()
    }

    private fun empty() = CoverSnapshot(riskScore = 0, band = "CALM", activeCovers = 0, unreadAlerts = 0)
}
