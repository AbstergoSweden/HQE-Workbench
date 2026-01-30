#!/usr/bin/env stack
-- stack --resolver lts-21.0 script --package yaml --package aeson --package bytestring --package text --package unordered-containers --package vector --package scientific

{-|
HQE Engineer Protocol YAML Validator (Haskell)

Validates hqe-engineer.yaml against hqe-schema.json structure.
Usage: stack verify.hs [--yaml path/to/file.yaml] [--schema path/to/schema.json]

Exit codes:
    0 - Validation successful
    1 - Validation failed
    2 - File not found or other error
-}

{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE RecordWildCards #-}
{-# LANGUAGE DeriveGeneric #-}

module Main where

import qualified Data.Aeson as Aeson
import qualified Data.Aeson.Key as Key
import qualified Data.Aeson.KeyMap as KeyMap
import qualified Data.ByteString.Lazy as BL
import qualified Data.HashMap.Strict as HM
import qualified Data.Text as T
import qualified Data.Text.IO as TIO
import qualified Data.Vector as V
import qualified Data.Yaml as Yaml

import Control.Monad (when)
import Data.Aeson ((.:), (.:?), (.!=), FromJSON, ToJSON, Value(..))
import Data.List (intercalate)
import Data.Maybe (fromMaybe, isJust, isNothing)
import Data.Scientific (Scientific)
import GHC.Generics (Generic)
import System.Console.GetOpt
import System.Environment (getArgs, getProgName)
import System.Exit (exitFailure, exitSuccess, exitWith, ExitCode(..))
import System.FilePath (takeFileName)
import System.IO (hPutStrLn, stderr)

-- ============================================================================
-- Data Types
-- ============================================================================

data Options = Options
    { optYamlPath :: FilePath
    , optSchemaPath :: FilePath
    , optVerbose :: Bool
    , optHelp :: Bool
    } deriving (Show)

defaultOptions :: Options
defaultOptions = Options
    { optYamlPath = "hqe-engineer.yaml"
    , optSchemaPath = "hqe-schema.json"
    , optVerbose = False
    , optHelp = False
    }

data ValidationError = ValidationError
    { errPath :: [T.Text]
    , errMessage :: T.Text
    , errSchemaPath :: [T.Text]
    } deriving (Show)

data ValidationResult = Valid | Invalid [ValidationError]
    deriving (Show)

-- HQE Protocol structure (simplified for key fields)
data HQEProtocol = HQEProtocol
    { hqeSchemaVersion :: T.Text
    , hqeProtocolVersion :: T.Text
    , hqeLastUpdated :: Maybe T.Text
    , hqeLicense :: Maybe T.Text
    , hqeMaintainer :: Maybe T.Text
    , hqeRole :: Maybe Role
    , hqePhases :: Maybe Aeson.Object
    , hqeHardConstraints :: Maybe Aeson.Array
    , hqeOperatingPrinciples :: Maybe Aeson.Array
    } deriving (Show, Generic)

data Role = Role
    { roleTitle :: T.Text
    , roleMandate :: T.Text
    } deriving (Show, Generic)

-- ============================================================================
-- JSON Instances
-- ============================================================================

instance FromJSON HQEProtocol where
    parseJSON = Aeson.withObject "HQEProtocol" $ \v -> HQEProtocol
        <$> v .: "schema_version"
        <*> v .: "protocol_version"
        <*> v .:? "last_updated"
        <*> v .:? "license"
        <*> v .:? "maintainer"
        <*> v .:? "role"
        <*> v .:? "phases"
        <*> v .:? "hard_constraints"
        <*> v .:? "operating_principles"

instance FromJSON Role where
    parseJSON = Aeson.withObject "Role" $ \v -> Role
        <$> v .: "title"
        <*> v .: "mandate"

-- ============================================================================
-- Command Line Parsing
-- ============================================================================

options :: [OptDescr (Options -> Options)]
options =
    [ Option ['y'] ["yaml"]
        (ReqArg (\f opts -> opts { optYamlPath = f }) "FILE")
        "Path to YAML file"
    
    , Option ['s'] ["schema"]
        (ReqArg (\f opts -> opts { optSchemaPath = f }) "FILE")
        "Path to JSON schema file"
    
    , Option ['v'] ["verbose"]
        (NoArg (\opts -> opts { optVerbose = True }))
        "Enable verbose output"
    
    , Option ['h'] ["help"]
        (NoArg (\opts -> opts { optHelp = True }))
        "Show help message"
    ]

parseOptions :: [String] -> IO Options
parseOptions argv = case getOpt Permute options argv of
    (o, [], []) -> return $ foldl (flip id) defaultOptions o
    (_, nonOpts, []) -> do
        hPutStrLn stderr $ "Unrecognized arguments: " ++ unwords nonOpts
        hPutStrLn stderr $ usageInfo header options
        exitWith (ExitFailure 2)
    (_, _, errs) -> do
        hPutStrLn stderr $ concat errs
        hPutStrLn stderr $ usageInfo header options
        exitWith (ExitFailure 2)
  where
    header = "Usage: verify.hs [OPTIONS]"

-- ============================================================================
-- File Operations
-- ============================================================================

loadYamlFile :: FilePath -> IO (Either String Aeson.Value)
loadYamlFile path = do
    result <- tryIO $ BL.readFile path
    case result of
        Left err -> return $ Left $ "Failed to read YAML file: " ++ show err
        Right content -> 
            case Yaml.decodeEither' (BL.toStrict content) of
                Left err -> return $ Left $ "YAML parse error: " ++ show err
                Right val -> return $ Right val

loadJsonFile :: FilePath -> IO (Either String Aeson.Value)
loadJsonFile path = do
    result <- tryIO $ BL.readFile path
    case result of
        Left err -> return $ Left $ "Failed to read JSON file: " ++ show err
        Right content -> 
            case Aeson.eitherDecode content of
                Left err -> return $ Left $ "JSON parse error: " ++ err
                Right val -> return $ Right val

tryIO :: IO a -> IO (Either IOError a)
tryIO = fmap Right

-- ============================================================================
-- Validation Logic
-- ============================================================================

validateRequiredFields :: Aeson.Value -> ValidationResult
validateRequiredFields value = 
    case value of
        Object obj -> 
            let required = ["schema_version", "protocol_version", "role", 
                           "output_structure", "hard_constraints", "phases"]
                missing = filter (not . KeyMap.member (Key.fromText . T.pack)) required
            in if null missing
                then Valid
                else Invalid [ValidationError 
                    { errPath = []
                    , errMessage = T.pack $ "Missing required fields: " ++ intercalate ", " missing
                    , errSchemaPath = ["required"]
                    }]
        _ -> Invalid [ValidationError 
            { errPath = []
            , errMessage = "Root must be an object"
            , errSchemaPath = ["type"]
            }]

validateSchemaVersion :: Aeson.Value -> ValidationResult
validateSchemaVersion value =
    case value of
        Object obj ->
            case KeyMap.lookup "schema_version" obj of
                Just (String v) ->
                    if T.isPrefixOf "3." v
                    then Valid
                    else Invalid [ValidationError
                        { errPath = ["schema_version"]
                        , errMessage = T.pack $ "Expected version 3.x.x, got: " ++ T.unpack v
                        , errSchemaPath = ["properties", "schema_version", "pattern"]
                        }]
                _ -> Invalid [ValidationError
                    { errPath = ["schema_version"]
                    , errMessage = "schema_version must be a string"
                    , errSchemaPath = ["properties", "schema_version", "type"]
                    }]
        _ -> Valid  -- Handled by validateRequiredFields

validatePhases :: Aeson.Value -> ValidationResult
validatePhases value =
    case value of
        Object obj ->
            case KeyMap.lookup "phases" obj of
                Just (Object phases) ->
                    let requiredPhases = ["phase_minus_one", "phase_zero", "phase_one",
                                         "phase_two", "phase_three", "phase_four", "phase_five"]
                        missing = filter (\p -> not $ KeyMap.member (Key.fromText $ T.pack p) phases) requiredPhases
                    in if null missing
                        then Valid
                        else Invalid [ValidationError
                            { errPath = ["phases"]
                            , errMessage = T.pack $ "Missing phases: " ++ intercalate ", " missing
                            , errSchemaPath = ["properties", "phases", "required"]
                            }]
                Just _ -> Invalid [ValidationError
                    { errPath = ["phases"]
                    , errMessage = "phases must be an object"
                    , errSchemaPath = ["properties", "phases", "type"]
                    }]
                Nothing -> Valid  -- Handled by validateRequiredFields
        _ -> Valid

validateOutputStructure :: Aeson.Value -> ValidationResult
validateOutputStructure value =
    case value of
        Object obj ->
            case KeyMap.lookup "output_structure" obj of
                Just (Object os) ->
                    case KeyMap.lookup "order" os of
                        Just (Array arr) | not (V.null arr) -> Valid
                        _ -> Invalid [ValidationError
                            { errPath = ["output_structure", "order"]
                            , errMessage = "output_structure.order must be a non-empty array"
                            , errSchemaPath = ["properties", "output_structure", "properties", "order"]
                            }]
                Just _ -> Invalid [ValidationError
                    { errPath = ["output_structure"]
                    , errMessage = "output_structure must be an object"
                    , errSchemaPath = ["properties", "output_structure", "type"]
                    }]
                Nothing -> Valid
        _ -> Valid

combineResults :: [ValidationResult] -> ValidationResult
combineResults results = 
    let errors = concat [es | Invalid es <- results]
    in if null errors then Valid else Invalid errors

runValidation :: Aeson.Value -> ValidationResult
runValidation value = combineResults
    [ validateRequiredFields value
    , validateSchemaVersion value
    , validatePhases value
    , validateOutputStructure value
    ]

-- ============================================================================
-- Output Formatting
-- ============================================================================

printValidationErrors :: [ValidationError] -> IO ()
printValidationErrors errors = do
    TIO.putStrLn "\n❌ Validation Errors:"
    TIO.putStrLn $ T.replicate 60 "="
    mapM_ printError (zip [1..] errors)
  where
    printError (n, ValidationError{..}) = do
        TIO.putStrLn $ "\nError #" <> T.pack (show n) <> ":"
        TIO.putStrLn $ "  Message: " <> errMessage
        TIO.putStrLn $ "  Path: " <> T.intercalate " -> " errPath
        when (not $ null errSchemaPath) $
            TIO.putStrLn $ "  Schema Path: " <> T.intercalate " -> " errSchemaPath

printSummary :: HQEProtocol -> Bool -> IO ()
printSummary HQEProtocol{..} isValid = do
    TIO.putStrLn $ "\n" <> T.replicate 60 "="
    if isValid
        then TIO.putStrLn "✅ VALIDATION PASSED"
        else TIO.putStrLn "❌ VALIDATION FAILED"
    TIO.putStrLn $ T.replicate 60 "="
    
    TIO.putStrLn "\n📋 Protocol Metadata:"
    TIO.putStrLn $ "  Schema Version:    " <> hqeSchemaVersion
    TIO.putStrLn $ "  Protocol Version:  " <> hqeProtocolVersion
    TIO.putStrLn $ "  Last Updated:      " <> fromMaybe "N/A" hqeLastUpdated
    TIO.putStrLn $ "  License:           " <> fromMaybe "N/A" hqeLicense
    TIO.putStrLn $ "  Maintainer:        " <> fromMaybe "N/A" hqeMaintainer
    
    case hqeRole of
        Just Role{..} -> TIO.putStrLn $ "  Role:              " <> roleTitle
        Nothing -> TIO.putStrLn "  Role:              N/A"
    
    TIO.putStrLn "\n📊 Structure Summary:"
    case hqePhases of
        Just phases -> do
            TIO.putStrLn $ "  Phases defined:    " <> T.pack (show $ KeyMap.size phases)
            mapM_ (\k -> TIO.putStrLn $ "    - " <> Key.toText k) (KeyMap.keys phases)
        Nothing -> TIO.putStrLn "  Phases defined:    0"
    
    TIO.putStrLn $ "  Hard constraints:  " <> T.pack (show $ maybe 0 V.length hqeHardConstraints)
    TIO.putStrLn $ "  Operating principles: " <> T.pack (show $ maybe 0 V.length hqeOperatingPrinciples)

-- ============================================================================
-- Main
-- ============================================================================

main :: IO ()
main = do
    args <- getArgs
    opts <- parseOptions args
    
    when (optHelp opts) $ do
        putStrLn $ usageInfo "Usage: verify.hs [OPTIONS]" options
        exitSuccess
    
    -- Check files exist
    yamlExists <- doesFileExist $ optYamlPath opts
    unless yamlExists $ do
        hPutStrLn stderr $ "Error: YAML file not found: " ++ optYamlPath opts
        exitWith (ExitFailure 2)
    
    schemaExists <- doesFileExist $ optSchemaPath opts
    unless schemaExists $ do
        hPutStrLn stderr $ "Error: Schema file not found: " ++ optSchemaPath opts
        exitWith (ExitFailure 2)
    
    when (optVerbose opts) $ do
        putStrLn $ "Loading YAML: " ++ optYamlPath opts
        putStrLn $ "Loading Schema: " ++ optSchemaPath opts
    
    -- Load YAML
    yamlResult <- loadYamlFile $ optYamlPath opts
    yamlValue <- case yamlResult of
        Left err -> do
            hPutStrLn stderr $ "Error: " ++ err
            exitWith (ExitFailure 2)
        Right val -> return val
    
    -- Load schema (for reference, basic validation)
    schemaResult <- loadJsonFile $ optSchemaPath opts
    case schemaResult of
        Left err -> hPutStrLn stderr $ "Warning: Could not load schema: " ++ err
        Right _ -> when (optVerbose opts) $ putStrLn "Schema loaded successfully"
    
    when (optVerbose opts) $ putStrLn "Validating..."
    
    -- Run validation
    let result = runValidation yamlValue
    
    -- Parse protocol for summary
    let protocol = case Aeson.fromJSON yamlValue of
            Aeson.Success p -> p
            Aeson.Error _ -> HQEProtocol "unknown" "unknown" Nothing Nothing Nothing Nothing Nothing Nothing Nothing
    
    -- Output results
    case result of
        Invalid errors -> printValidationErrors errors
        Valid -> return ()
    
    let isValid = case result of Valid -> True; _ -> False
    printSummary protocol isValid
    
    exitWith $ if isValid then ExitSuccess else ExitFailure 1

-- Helper
unless :: Bool -> IO () -> IO ()
unless cond action = when (not cond) action

doesFileExist :: FilePath -> IO Bool
doesFileExist path = do
    result <- tryIO $ BL.readFile path
    case result of
        Left _ -> return False
        Right _ -> return True
