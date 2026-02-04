interface DataViewerProps {
  datasetId?: string;
}

export function DataViewer({ datasetId }: DataViewerProps) {
  return (
    <div className="flex h-full min-h-[calc(100vh-2rem)] flex-col">
      <div className="grid h-full flex-1 grid-cols-[minmax(0,1fr)_minmax(0,2fr)]">
        <section className="border-r p-4">
          <h1 className="text-lg font-semibold">Datasets</h1>
          <p className="text-muted-foreground mt-1 text-sm">
            Dataset table placeholder (search, tags, favorites, status).
          </p>
        </section>
        <section className="p-4">
          {datasetId ? (
            <>
              <h2 className="text-lg font-semibold">Dataset {datasetId}</h2>
              <p className="text-muted-foreground mt-1 text-sm">
                Detail + charts placeholder.
              </p>
            </>
          ) : (
            <>
              <h2 className="text-lg font-semibold">No dataset selected</h2>
              <p className="text-muted-foreground mt-1 text-sm">
                Choose a dataset to view charts and metadata.
              </p>
            </>
          )}
        </section>
      </div>
    </div>
  );
}
