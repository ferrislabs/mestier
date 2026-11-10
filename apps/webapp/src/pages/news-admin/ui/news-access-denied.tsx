export function NewsAccessDenied() {
	return (
		<div className="mx-auto flex max-w-md flex-col gap-2 p-10 text-center">
			<h1 className="font-semibold text-lg">Accès réservé</h1>
			<p className="text-muted-foreground text-sm">
				Le back-office des news est réservé aux administrateurs de Mestier.
			</p>
		</div>
	)
}
