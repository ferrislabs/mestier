import { FileUp, Link2, LoaderCircle } from 'lucide-react'
import { useId, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'

export function SpecSourcePicker({
	initialUrl = '',
	loading,
	submitLabel,
	onLoadUrl,
	onLoadFile,
	onSourceChange,
}: {
	initialUrl?: string
	loading: boolean
	submitLabel: string
	onLoadUrl: (url: string) => void
	onLoadFile: (file: File) => void
	onSourceChange: () => void
}) {
	const urlId = useId()
	const fileId = useId()
	const [url, setUrl] = useState(initialUrl)
	const loadFile = (file: File | undefined) => {
		if (file) onLoadFile(file)
	}

	return (
		<Tabs defaultValue="url" onValueChange={onSourceChange}>
			<TabsList>
				<TabsTrigger value="url">
					<Link2 />
					Depuis une URL
				</TabsTrigger>
				<TabsTrigger value="file">
					<FileUp />
					Fichier
				</TabsTrigger>
			</TabsList>
			<TabsContent value="url" className="mt-3">
				<form
					className="grid gap-2"
					onSubmit={(event) => {
						event.preventDefault()
						if (url.trim()) onLoadUrl(url.trim())
					}}
				>
					<Label htmlFor={urlId}>URL de la spec</Label>
					<div className="flex gap-2">
						<Input
							id={urlId}
							type="url"
							value={url}
							onChange={(event) => setUrl(event.target.value)}
							placeholder="https://api.exemple.fr/openapi.json"
							autoFocus
						/>
						<Button type="submit" disabled={!url.trim() || loading}>
							{loading && <LoaderCircle className="animate-spin" />}
							{submitLabel}
						</Button>
					</div>
				</form>
			</TabsContent>
			<TabsContent value="file" className="mt-3">
				<label
					htmlFor={fileId}
					className="flex cursor-pointer flex-col items-center gap-2 rounded-md border-2 border-border border-dashed px-4 py-8 text-center text-muted-foreground text-sm transition-colors hover:border-ring hover:text-foreground"
					onDragOver={(event) => event.preventDefault()}
					onDrop={(event) => {
						event.preventDefault()
						loadFile(event.dataTransfer.files[0])
					}}
				>
					<FileUp className="size-6" aria-hidden />
					<span>
						Déposez un fichier <code className="font-mono">.json</code>,{' '}
						<code className="font-mono">.yaml</code> ou{' '}
						<code className="font-mono">.yml</code>, ou cliquez pour le choisir.
					</span>
				</label>
				<input
					id={fileId}
					type="file"
					accept=".json,.yaml,.yml,application/json"
					className="sr-only"
					onChange={(event) => loadFile(event.target.files?.[0])}
				/>
			</TabsContent>
		</Tabs>
	)
}
