import Foundation
import RoutiaCore
import SwiftData
import SwiftUI

/// Une habitude enregistrée sur l'iPhone (SwiftData). Les règles des séries viennent de
/// `RoutiaCore`, les mêmes que sur le site.
@Model
final class HabitRecord {
    var id: UUID = UUID()
    var name: String = ""
    /// Le nombre de fois par semaine : 7, c'est chaque jour.
    var frequency: Int = 7
    var colorIndex: Int = 0
    /// `AAAA-MM-JJ`.
    var createdKey: String = ""
    /// Les jours faits, `AAAA-MM-JJ`.
    var doneKeys: [String] = []
    var createdAt: Date = Date()

    init(name: String, frequency: Int, colorIndex: Int, today: Day = .today) {
        self.name = HabitState.clean(name)
        self.frequency = min(max(frequency, 1), 7)
        self.colorIndex = colorIndex
        self.createdKey = today.key
    }

    var state: HabitState {
        HabitState(
            name: name,
            frequency: frequency,
            created: Day(key: createdKey) ?? .today,
            done: Set(doneKeys.compactMap(Day.init(key:)))
        )
    }

    func toggle(_ day: Day) {
        var s = state
        s.toggle(day)
        doneKeys = s.done.map(\.key).sorted()
        createdKey = s.created.key
    }

    var color: Color { Palette.color(colorIndex) }
}

enum Palette {
    static let hex: [UInt32] = [0x7C5CFF, 0x22D3EE, 0x34D399, 0xFBBF24, 0xF472B6, 0xFB7185]

    static func color(_ index: Int) -> Color {
        let v = hex[((index % hex.count) + hex.count) % hex.count]
        return Color(
            red: Double((v >> 16) & 0xFF) / 255,
            green: Double((v >> 8) & 0xFF) / 255,
            blue: Double(v & 0xFF) / 255
        )
    }

    static let background = Color(red: 11 / 255, green: 8 / 255, blue: 32 / 255)
    static let gradient = LinearGradient(
        colors: [color(0), color(1), color(4)],
        startPoint: .topLeading,
        endPoint: .bottomTrailing
    )
}
