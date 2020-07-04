import * as React from "react";
import { useCallback } from "react";
import ReactiveElements from "reactive-elements";
import GraphiQL from "graphiql";
import GraphiQLExplorer from "graphiql-explorer";
import 'babel-polyfill';

interface EditorProps {
    schemaUrl: string
}

const GraphQLEditor = ({ schemaUrl }: EditorProps) => {
    const fetcher = async (object) => { return JSON.parse("{}") };

    const graphQLFetcher = useCallback(
        graphQLParams =>
            fetch(schemaUrl, {
                method: 'post',
                headers: { 'Content-Type': 'application/json', 'Authorization': 'Bearer ' },
                body: JSON.stringify(graphQLParams),
            })
                .then(response => response.json())
                .catch(response => response.text()),
        [schemaUrl]
    );

    const style = `
      :host {
        all: initial;
        display: block;
      }
    `;

    return <>
        <style>{style}</style>
        <link href="https://unpkg.com/graphiql/graphiql.min.css" rel="stylesheet" />
        <GraphiQL fetcher={graphQLFetcher}></GraphiQL>
    </>
}

GraphQLEditor.attributeChanged = (name, oldValue, newValue) => {
    console.log("attr changed")
}

ReactiveElements('gql-editor', GraphQLEditor, { useShadowDom: true });