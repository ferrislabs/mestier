import { TextField } from '#/components/reference-table'
import { Label } from '#/components/ui/label'
import { Switch } from '#/components/ui/switch'
import type { MemberFormValues } from '#/pages/hr/types'

export function MemberFormFields({
	idPrefix,
	values,
	onChange,
}: {
	idPrefix: string
	values: MemberFormValues
	onChange: (patch: Partial<MemberFormValues>) => void
}) {
	return (
		<>
			<div className="grid grid-cols-1 gap-4 md:grid-cols-3">
				<TextField
					label="Nom"
					value={values.lastName}
					onChange={(lastName) => onChange({ lastName })}
				/>
				<TextField
					label="Prénom"
					value={values.firstName}
					onChange={(firstName) => onChange({ firstName })}
					placeholder="Optionnel"
				/>
				<TextField
					label="Taux horaire"
					value={values.hourlyRate}
					onChange={(hourlyRate) => onChange({ hourlyRate })}
					inputMode="decimal"
					suffix="€/h"
					placeholder="Optionnel"
					disabled={values.isSalaried}
				/>
				{values.isSalaried ? (
					<TextField
						label="Coût employeur mensuel"
						value={values.monthlyCost}
						onChange={(monthlyCost) => onChange({ monthlyCost })}
						inputMode="decimal"
						suffix="€/mois"
						placeholder="Salaire chargé"
					/>
				) : null}
			</div>
			<div className="mt-4 flex items-center gap-2">
				<Switch
					id={`${idPrefix}-salaried`}
					checked={values.isSalaried}
					onCheckedChange={(isSalaried) =>
						onChange({ isSalaried, hourlyRate: '', monthlyCost: '' })
					}
				/>
				<Label htmlFor={`${idPrefix}-salaried`} className="text-sm font-normal">
					Salarié (coût mensuel plutôt qu'horaire)
				</Label>
			</div>
			{values.isSalaried ? (
				<p className="mt-2 text-xs text-muted-foreground">
					Le coût horaire est calculé à partir de ce montant et des heures
					contractuelles. Sans montant, la rentabilité refuse de chiffrer son
					temps plutôt que de le compter gratuit.
				</p>
			) : null}
		</>
	)
}
