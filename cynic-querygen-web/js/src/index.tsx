import * as React from "react";
import { useCallback, useState, useEffect } from "react";
import ReactiveElements from "terryoy-reactive-elements";
import GraphiQL from "graphiql";
import GraphiQLExplorer from "graphiql-explorer";
import "babel-polyfill";
import {
  getIntrospectionQuery,
  buildClientSchema,
  GraphQLSchema,
  printSchema,
  buildSchema,
} from "graphql";

import GeneratedRustViewer from "./GeneratedRustViewer";
import { FetcherParams, FetcherOpts } from "graphiql/dist/components/GraphiQL";

interface EditorProps {
  schemaUrl: string;
  container: HTMLElement;
  generatedCode: string;
  schema?: string;
}

const GraphQLEditor = (props: EditorProps) => {
  const { schemaUrl, container } = props;

  const generatedCode = props.generatedCode.replace(/&NL;/g, "\n");

  const [query, setQuery] = useState<string | undefined>();
  const [headers, setHeaders] = useState<{ string: string } | undefined>();
  const [schema, setSchema] = useState<GraphQLSchema | undefined>();
  const [explorerOpen, setExplorerOpen] = useState(true);

  const graphQLFetcher = useCallback(makeFetcher(schemaUrl), [schemaUrl]);

  useEffect(() => {
    if (schemaUrl && schemaUrl.length != 0) {
      const handler = async () => {
        const result = await graphQLFetcher(
          { query: getIntrospectionQuery(), operationName: null },
          { shouldPersistHeaders: false, headers: headers }
        );
        const clientSchema = buildClientSchema(result.data);

        setSchema(clientSchema);

        container.dispatchEvent(
          new CustomEvent("schema-loaded", {
            bubbles: true,
            detail: printSchema(clientSchema),
          })
        );
      };

      handler();
    } else if (props.schema && props.schema.length != 0) {
      const parsedSchema = buildSchema(props.schema.replace(/&NL;/g, "\n"));
      setSchema(parsedSchema);
    }
  }, [schemaUrl, props.schema, headers, container, setSchema]);

  const onEditQuery = (query: string) => {
    container.dispatchEvent(
      new CustomEvent("change", { bubbles: true, detail: query })
    );
    setQuery(query);
  };

  const onEditHeaders = (headers: string) => {
    try {
      setHeaders(JSON.parse(headers));
    } catch (e) {
      // Do nothing, whatever
    }
  };

  return (
    <>
      <style>
        {`
        :host {
            all: initial;
            display: block;
        }
      `}
      </style>

      <link
        href="https://unpkg.com/graphiql/graphiql.min.css"
        rel="stylesheet"
      />
      <div className="graphiql-container">
        <GraphiQLExplorer
          query={query}
          schema={schema}
          onEdit={onEditQuery}
          explorerIsOpen={explorerOpen}
          onToggleExplorer={() => setExplorerOpen(!explorerOpen)}
        />
        <GraphiQL
          fetcher={graphQLFetcher}
          schema={schema}
          headerEditorEnabled
          query={query}
          onEditQuery={onEditQuery}
          onEditHeaders={onEditHeaders}
        >
          <GraphiQL.Logo>Query Builder</GraphiQL.Logo>
          <GraphiQL.Toolbar></GraphiQL.Toolbar>
          <GraphiQL.Footer>
            <GeneratedRustViewer
              generatedCode={generatedCode}
            ></GeneratedRustViewer>
          </GraphiQL.Footer>
        </GraphiQL>
      </div>
    </>
  );
};

ReactiveElements("gql-editor", GraphQLEditor, { useShadowDom: true });

const makeFetcher = (schemaUrl: string) => {
  return async (graphQLParams: FetcherParams, opts: FetcherOpts) => {
    if (schemaUrl.length === 0) {
      return "Can't run queries - you pasted a schema so we don't have a URL";
    }

    const response = await fetch(schemaUrl, {
      method: "post",
      headers: {
        "Content-Type": "application/json",
        ...opts.headers,
      },
      body: JSON.stringify(graphQLParams),
    });

    const responseBody = await response.text();

    try {
      return JSON.parse(responseBody);
    } catch (e) {
      return responseBody;
    }
  };
};
