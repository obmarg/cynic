import * as React from "react";
import { useCallback, useState, useEffect } from "react";
import ReactiveElements from "reactive-elements";
import GraphiQL from "graphiql";
import GraphiQLExplorer from "graphiql-explorer";
import "babel-polyfill";
import {
  getIntrospectionQuery,
  buildClientSchema,
  GraphQLSchema,
} from "graphql";

import GeneratedRustViewer from "./GeneratedRustViewer";

interface EditorProps {
  schemaUrl: string;
  container: HTMLElement;
}

const GraphQLEditor = (props: EditorProps) => {
  const { schemaUrl, container } = props;

  const [query, setQuery] = useState<string | undefined>();
  const [schema, setSchema] = useState<GraphQLSchema | undefined>();
  const [explorerOpen, setExplorerOpen] = useState(true);
  const graphQLFetcher = useCallback(makeFetcher(schemaUrl), [schemaUrl]);

  useEffect(() => {
    const handler = async () => {
      const result = await graphQLFetcher({ query: getIntrospectionQuery() });
      setSchema(buildClientSchema(result.data));
    };

    handler();
  }, [schemaUrl]);

  const onEditQuery = (query: string) => {
    container.dispatchEvent(
      new CustomEvent("change", { bubbles: true, detail: query })
    );
    setQuery(query);
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
        >
          <GraphiQL.Logo>Query Builder</GraphiQL.Logo>
          <GraphiQL.Toolbar></GraphiQL.Toolbar>
          <GraphiQL.Footer>
            <GeneratedRustViewer></GeneratedRustViewer>
          </GraphiQL.Footer>
        </GraphiQL>
      </div>
    </>
  );
};

ReactiveElements("gql-editor", GraphQLEditor, { useShadowDom: true });

const makeFetcher = (schemaUrl) => {
  return (graphQLParams) =>
    fetch(schemaUrl, {
      method: "post",
      headers: {
        "Content-Type": "application/json",
        Authorization:
          "Bearer ",
      },
      body: JSON.stringify(graphQLParams),
    })
      .then(function (response) {
        return response.text();
      })
      .then(function (responseBody) {
        try {
          return JSON.parse(responseBody);
        } catch (e) {
          return responseBody;
        }
      });
};
