import * as React from "react";
import ReactiveElements from "reactive-elements";
import GraphiQL from "graphiql";
import GraphiQLExplorer from "graphiql-explorer";
import 'babel-polyfill';

const GraphQLEditor: React.FC = () => {
    const fetcher = async (object) => { return JSON.parse("{}") };

    return <>
        <link href="https://unpkg.com/graphiql/graphiql.min.css" rel="stylesheet" />
        <GraphiQL fetcher={fetcher}></GraphiQL>
    </>
}

ReactiveElements('gql-editor', GraphQLEditor, { useShadowDom: true });