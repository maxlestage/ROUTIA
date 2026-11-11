import Testing
@testable import RoutiaCore

private func day(_ key: String) -> Day { Day(key: key)! }

@Suite("Dates")
struct DateTests {
    @Test func roundTrip() {
        #expect(Day(year: 1970, month: 1, day: 1).index == 0)
        #expect(day("2026-11-10").key == "2026-11-10")
        #expect(day("2024-02-29").adding(1).key == "2024-03-01")
        #expect(Day(key: "2025-02-29") == nil)
        #expect(Day(key: "hier") == nil)
        for n in stride(from: -800_000, to: 800_000, by: 997) {
            let (y, m, d) = Day(index: n).ymd
            #expect(Day(year: y, month: m, day: d).index == n)
        }
    }

    @Test func weeksStartOnMonday() {
        #expect(day("2026-11-10").weekday == 1)
        #expect(day("2026-11-15").weekStart == day("2026-11-09"))
    }
}

@Suite("Séries")
struct StreakTests {
    @Test func daily() {
        let today = day("2026-11-10")
        var h = HabitState(name: "  Méditer   10 min ", frequency: 7, created: today.adding(-10))
        #expect(h.name == "Méditer 10 min")
        for i in 1...3 { h.toggle(today.adding(-i)) }
        #expect(h.streak(today: today) == 3)
        #expect(h.cell(today, today: today) == .pending)
        h.toggle(today)
        #expect(h.streak(today: today) == 4)
        h.toggle(today.adding(-2))
        #expect(h.streak(today: today) == 2)
        #expect(h.cell(today.adding(-2), today: today) == .missed)
        h.toggle(today.adding(-2))
        #expect(h.bestStreak(today: today) == 4)
        #expect(h.missed(today: today) == 7)
    }

    @Test func weekly() {
        let today = day("2026-11-12")
        var h = HabitState(name: "Courir", frequency: 3, created: day("2026-10-19"))
        for monday in ["2026-10-19", "2026-10-26", "2026-11-02"] {
            for i in [0, 2, 4] { h.toggle(day(monday).adding(i)) }
        }
        #expect(h.streak(today: today) == 3)
        #expect(h.unit.label(3) == "semaines")
        #expect(h.cell(day("2026-10-20"), today: today) == .rest)
        h.toggle(day("2026-10-28"))
        #expect(h.streak(today: today) == 1) // seule la semaine du 2 novembre tient
        #expect(h.cell(day("2026-10-27"), today: today) == .missed)
    }

    @Test func export() {
        let today = day("2026-11-10")
        var h = HabitState(name: "Méditer", frequency: 7, created: today.adding(-5))
        h.toggle(today)
        h.toggle(today.adding(-1))
        let text = exportText([h], today: today)
        #expect(text.hasPrefix("Routia — bilan du 10 nov. 2026"))
        #expect(text.contains("Série : 2 jours"))
        #expect(exportText([], today: today).contains("Aucune habitude"))
    }
}
