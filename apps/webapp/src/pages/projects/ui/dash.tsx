export function Dash({ label = 'Non renseigné' }: { label?: string }) {
	return (
		<span className="text-muted-foreground">
			<span aria-hidden>—</span>
			<span className="sr-only">{label}</span>
		</span>
	)
}
