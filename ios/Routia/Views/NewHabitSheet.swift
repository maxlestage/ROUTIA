import RoutiaCore
import SwiftUI

/// Créer une habitude : un nom, une fréquence, une couleur.
struct NewHabitSheet: View {
    let nextColor: Int
    let create: (String, Int, Int) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var frequency = 7
    @State private var color = 0
    @FocusState private var focused: Bool

    private let frequencies = [7, 5, 3, 1]

    var body: some View {
        NavigationStack {
            Form {
                Section("Nom") {
                    TextField("Méditer, lire, courir…", text: $name)
                        .focused($focused)
                        .submitLabel(.done)
                        .onSubmit(save)
                }
                Section("Fréquence") {
                    Picker("Fréquence", selection: $frequency) {
                        ForEach(frequencies, id: \.self) { n in
                            Text(n == 7 ? "Chaque jour" : "\(n)×/sem.").tag(n)
                        }
                    }
                    .pickerStyle(.segmented)
                }
                Section("Couleur") {
                    HStack {
                        ForEach(Palette.hex.indices, id: \.self) { i in
                            Circle()
                                .fill(Palette.color(i))
                                .frame(width: 36, height: 36)
                                .overlay { Circle().strokeBorder(.white, lineWidth: color == i ? 3 : 0) }
                                .scaleEffect(color == i ? 1.12 : 1)
                                .animation(.bouncy, value: color)
                                .onTapGesture { color = i }
                                .accessibilityLabel("Couleur \(i + 1)")
                                .accessibilityAddTraits(color == i ? [.isButton, .isSelected] : .isButton)
                            if i < Palette.hex.count - 1 { Spacer(minLength: 0) }
                        }
                    }
                }
            }
            .scrollContentBackground(.hidden)
            .navigationTitle("Nouvelle habitude")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Annuler", role: .cancel) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Créer", action: save)
                        .disabled(HabitState.clean(name).isEmpty)
                }
            }
            .onAppear {
                color = nextColor % Palette.hex.count
                focused = true
            }
        }
    }

    private func save() {
        let clean = HabitState.clean(name)
        guard !clean.isEmpty else { return }
        create(clean, frequency, color)
        dismiss()
    }
}
