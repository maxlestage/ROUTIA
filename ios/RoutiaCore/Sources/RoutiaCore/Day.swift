import Foundation

/// Un jour du calendrier, compté depuis le 1er janvier 1970.
public struct Day: Hashable, Comparable, Sendable, Codable, CustomStringConvertible {
    public let index: Int

    public init(index: Int) {
        self.index = index
    }

    /// Le jour d'une date du calendrier grégorien (algorithme « days_from_civil »).
    public init(year: Int, month: Int, day: Int) {
        let y = month <= 2 ? year - 1 : year
        let era = (y >= 0 ? y : y - 399) / 400
        let yoe = y - era * 400
        let mp = month > 2 ? month - 3 : month + 9
        let doy = (153 * mp + 2) / 5 + day - 1
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy
        index = era * 146_097 + doe - 719_468
    }

    /// Le jour de `date` dans le fuseau du calendrier.
    public init(_ date: Date, calendar: Calendar = .current) {
        let c = calendar.dateComponents([.year, .month, .day], from: date)
        self.init(year: c.year ?? 1970, month: c.month ?? 1, day: c.day ?? 1)
    }

    /// `AAAA-MM-JJ`.
    public init?(key: String) {
        let parts = key.split(separator: "-")
        guard parts.count == 3,
              let y = Int(parts[0]), let m = Int(parts[1]), let d = Int(parts[2]),
              (1...12).contains(m), d >= 1, d <= Day.daysIn(month: m, year: y)
        else { return nil }
        self.init(year: y, month: m, day: d)
    }

    public static var today: Day { Day(Date()) }

    public var ymd: (year: Int, month: Int, day: Int) {
        let z = index + 719_468
        let era = (z >= 0 ? z : z - 146_096) / 146_097
        let doe = z - era * 146_097
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100)
        let mp = (5 * doy + 2) / 153
        let d = doy - (153 * mp + 2) / 5 + 1
        let m = mp < 10 ? mp + 3 : mp - 9
        return (yoe + era * 400 + (m <= 2 ? 1 : 0), m, d)
    }

    public var key: String {
        let (y, m, d) = ymd
        return String(format: "%04d-%02d-%02d", y, m, d)
    }

    public var description: String { key }

    public func adding(_ days: Int) -> Day { Day(index: index + days) }

    /// 0 pour lundi … 6 pour dimanche (le 1er janvier 1970 était un jeudi).
    public var weekday: Int { ((index + 3) % 7 + 7) % 7 }

    /// Le lundi de la semaine.
    public var weekStart: Day { adding(-weekday) }

    public static func < (a: Day, b: Day) -> Bool { a.index < b.index }

    static func daysIn(month: Int, year: Int) -> Int {
        switch month {
        case 2: (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 ? 29 : 28
        case 4, 6, 9, 11: 30
        default: 31
        }
    }

    public static let weekdaySymbols = ["L", "M", "M", "J", "V", "S", "D"]
    public static let monthSymbols = [
        "janv.", "févr.", "mars", "avr.", "mai", "juin",
        "juil.", "août", "sept.", "oct.", "nov.", "déc.",
    ]
}
