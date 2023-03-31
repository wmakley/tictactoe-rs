-- Make a GET request to load a book called "Public Opinion"
--
-- Read how it works:
--   https://guide.elm-lang.org/effects/http.html
--


port module App exposing (Model(..), Msg(..), init, main, subscriptions, update, view)

import Browser
import Html exposing (Html, pre, text)
import Html.Attributes exposing (type_)
import Html.Events
import Json.Decode as Decode
import Json.Encode as Encode


port socketSend : Encode.Value -> Cmd msg


port socketReceive : (Decode.Value -> msg) -> Sub msg



-- MAIN


main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }



-- MODEL


type Model
    = Menu String
    | Game GameState
    | Error String


type alias GameState =
    { token : String
    , player1 : ()
    , player2 : ()
    , board : List String
    , chat : List ( Int, String )
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Menu "", Cmd.none )



-- UPDATE


type Msg
    = JoinTokenChanged String
    | JoinGame
    | GameJoined String
    | UpdateGameState GameState
    | LeaveGame
    | ShowError String


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        JoinTokenChanged token ->
            case model of
                Menu _ ->
                    ( Menu token, Cmd.none )

                Game game ->
                    ( Game
                        { game
                            | token = token
                        }
                    , Cmd.none
                    )

                Error _ ->
                    ( Menu token, Cmd.none )

        JoinGame ->
            case model of
                Menu token ->
                    ( Menu token
                    , socketSend
                        (Encode.object
                            [ ( "msg", Encode.string "join" )
                            , ( "payload", Encode.string token )
                            ]
                        )
                    )

                _ ->
                    ( model, Cmd.none )

        GameJoined token ->
            ( Game { token = token, player1 = (), player2 = (), board = [], chat = [] }, Cmd.none )

        UpdateGameState state ->
            ( Game state, Cmd.none )

        LeaveGame ->
            ( Menu "", socketSend (Encode.object [ ( "msg", Encode.string "leave" ) ]) )

        ShowError e ->
            ( Error e, Cmd.none )



-- SUBSCRIPTIONS


type alias SocketMsg =
    { msg : String
    , payload : Decode.Value
    }


socketMsgDecoder : Decode.Decoder SocketMsg
socketMsgDecoder =
    Decode.map2 SocketMsg
        (Decode.field "msg" Decode.string)
        (Decode.field "payload" Decode.value)


gameStateDecoder : Decode.Decoder GameState
gameStateDecoder =
    Decode.map5 GameState
        (Decode.field "token" Decode.string)
        (Decode.field "player1" (Decode.succeed ()))
        (Decode.field "player2" (Decode.succeed ()))
        (Decode.field "board" (Decode.list Decode.string))
        (Decode.field "chat" (Decode.list (Decode.map2 Tuple.pair Decode.int Decode.string)))


subscriptions : Model -> Sub Msg
subscriptions model =
    socketReceive
        (\rawInput ->
            case Decode.decodeValue socketMsgDecoder rawInput of
                Ok { msg, payload } ->
                    case msg of
                        "gameJoined" ->
                            GameJoined
                                (Decode.decodeValue Decode.string payload
                                    |> Result.withDefault ""
                                )

                        _ ->
                            ShowError ("Unknown message: " ++ msg)

                Err e ->
                    ShowError ("Error decoding message: " ++ Debug.toString e)
        )



-- VIEW


view : Model -> Html Msg
view model =
    case model of
        Menu token ->
            Html.div []
                [ Html.input [ type_ "text", Html.Attributes.value token, Html.Events.onInput JoinTokenChanged ] []
                , Html.button [ Html.Events.onClick JoinGame ] [ Html.text "Join Game" ]
                ]

        Game state ->
            Html.div []
                [ Html.button [ Html.Events.onClick LeaveGame ] [ Html.text "Leave Game" ]
                , pre [] [ text <| Debug.toString state ]
                ]

        Error e ->
            Html.div []
                [ Html.text e
                ]
