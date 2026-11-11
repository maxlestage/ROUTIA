import Foundation

/// L'unité d'une série : des jours pour une habitude quotidienne, des semaines sinon.
public enum StreakUnit: Sendable {
    case days, weeks

    public func label(_ n: Int) -> String {
        switch (self, n) {
        case (.days, 0...1): "jour"
        case (.days, _): "jours"
        case (.weeks, 0...1): "semaine"
        case (.weeks, _): "semaines"
        }
    }
}

/// L'état d'une case de l'historique.
public enum DayCell: Sendable, Hashable {
    case done, missed, pending, rest, empty
}

/// Un jour de l'historique et son état.
public struct HistoryItem: Sendable, Hashable {
    public let day: Day
    public let cell: DayCell
}

/// Ce que les règles ont besoin de savoir d'une habitude.
public struct HabitState: Sendable, Equatable {
    public var name: String
    /// Le nombre de fois par semaine : 7, c'est chaque jour.
    public var frequency: Int
    public var created: Day
    public var done: Set<Day>

    public init(name: String, frequency: Int, created: Day, done: Set<Day> = []) {
        self.name = HabitState.clean(name)
        self.frequency = min(max(frequency, 1), 7)
        self.created = created
        self.done = done
    }

    /// Un nom propre : sans espaces en trop, 40 caractères au plus.
    public static func clean(_ name: String) -> String {
        String(name.split(whereSeparator: \.isWhitespace).joined(separator: " ").prefix(40))
    }

    public var isDaily: Bool { frequency >= 7 }
    public var unit: StreakUnit { isDaily ? .days : .weeks }

    public var frequencyLabel: String {
        isDaily ? "Chaque jour" : "\(frequency)× par semaine"
    }

    public func isDone(_ day: Day) -> Bool { done.contains(day) }

    /// Coche ou décoche ; cocher un jour d'avant la création la recule.
    public mutating func toggle(_ day: Day) {
        if done.contains(day) {
            done.remove(day)
        } else {
            done.insert(day)
            if day < created { created = day }
        }
    }

    public func doneInWeek(_ monday: Day) -> Int {
        (0..<7).filter { isDone(monday.adding($0)) }.count
    }

    /// La série en cours : un jour (ou une semaine) pas encore fini ne la casse pas.
    public func streak(today: Day) -> Int {
        if isDaily {
            var day = isDone(today) ? today : today.adding(-1)
            var n = 0
            while isDone(day) {
                n += 1
                day = day.adding(-1)
            }
            return n
        }
        let thisWeek = today.weekStart
        var week = doneInWeek(thisWeek) >= frequency ? thisWeek : thisWeek.adding(-7)
        var n = 0
        while week >= created.weekStart, doneInWeek(week) >= frequency {
            n += 1
            week = week.adding(-7)
        }
        return n
    }

    /// La plus longue série, de la création à aujourd'hui.
    public func bestStreak(today: Day) -> Int {
        var best = 0, run = 0
        if isDaily {
            var day = created
            while day <= today {
                if isDone(day) {
                    run += 1
                    best = max(best, run)
                } else if day != today {
                    run = 0
                }
                day = day.adding(1)
            }
        } else {
            var week = created.weekStart
            while week <= today {
                if doneInWeek(week) >= frequency {
                    run += 1
                    best = max(best, run)
                } else if week != today.weekStart {
                    run = 0
                }
                week = week.adding(7)
            }
        }
        return best
    }

    public func cell(_ day: Day, today: Day) -> DayCell {
        if isDone(day) { return .done }
        if day > today || day < created { return .empty }
        if day == today { return .pending }
        if isDaily { return .missed }
        let monday = day.weekStart
        return monday < today.weekStart && doneInWeek(monday) < frequency ? .missed : .rest
    }

    /// Les `days` derniers jours, du plus ancien à `today`.
    public func history(today: Day, days: Int) -> [HistoryItem] {
        (0..<days).reversed().map { i in
            let day = today.adding(-i)
            return HistoryItem(day: day, cell: cell(day, today: today))
        }
    }

    public func missed(today: Day, days: Int = 30) -> Int {
        history(today: today, days: days).filter { $0.cell == .missed }.count
    }

    /// La part des jours comptés qui ont été faits, en pourcents.
    public func rate(today: Day, days: Int = 30) -> Int {
        let cells = history(today: today, days: days).map(\.cell)
        let done = cells.filter { $0 == .done }.count
        let counted = done + cells.filter { $0 == .missed }.count
        return counted == 0 ? 0 : done * 100 / counted
    }
}

/// Le bilan en texte, comme sur le site.
public func exportText(_ habits: [HabitState], today: Day) -> String {
    let (y, m, d) = today.ymd
    var out = "Routia — bilan du \(d) \(Day.monthSymbols[m - 1]) \(y)\n\n"
    if habits.isEmpty { out += "Aucune habitude pour l'instant.\n" }
    for h in habits {
        let s = h.streak(today: today)
        let grid = h.history(today: today, days: 28).map { item -> Character in
            switch item.cell {
            case .done: "■"
            case .missed: "×"
            case .pending: "◇"
            case .rest: "·"
            case .empty: " "
            }
        }
        let line = String(grid).replacingOccurrences(of: "\\s+$", with: "", options: .regularExpression)
        out += "• \(h.name) (\(h.frequencyLabel.lowercased()))\n"
        out += "  Série : \(s) \(h.unit.label(s)) · record : \(h.bestStreak(today: today))"
        out += " · réussite (30 j) : \(h.rate(today: today)) % · manqués : \(h.missed(today: today))\n"
        out += "  \(line)\n\n"
    }
    out += "■ fait · × manqué · ◇ aujourd'hui · · repos\n"
    return out
}
