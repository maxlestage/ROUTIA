import SwiftData
import SwiftUI

@main
struct RoutiaApp: App {
    var body: some Scene {
        WindowGroup {
            TodayView()
                .preferredColorScheme(.dark)
        }
        .modelContainer(for: HabitRecord.self)
    }
}
