import ujson
import pandas as pd
from pathlib import Path
from dataclasses import dataclass
import jmespath
from evtx import PyEvtxParser
from typing import Any, Dict, Iterator, List, Tuple, Union


@dataclass
class JpValue:
    pattern_str: str
    compiled: jmespath.parser.ParsedResult

    def get_value(self, record: dict):
        return self.compiled.search(record)
    
    @classmethod
    def from_pattern(cls, pattern_str: str):
        compiled = jmespath.compile(pattern_str)
        return cls(pattern_str, compiled)
    

class Filter(JpValue):
    def matches(self, record: dict) -> bool:
        return bool(self.get_value(record))


@dataclass
class DocumentTransformer:
    mapping: Dict[str, JpValue]

    @staticmethod
    def empty():
        return DocumentTransformer(dict())

    def add_field(self, field: str, expression: str):
        if field in self.mapping:
            raise Exception(f"{field} already exists.")

        _c = jmespath.compile(expression)
        self.mapping[field] = JpValue(expression, _c)

    @staticmethod
    def from_fields(fields: Iterator[Tuple[str, str]]):
        d = DocumentTransformer.empty()
        for field, expression in fields:
            d.add_field(field, expression)

        return d

    def get_record(self, record: dict) -> dict:
        new_doc = {}
        for f, v in self.mapping.items():
            new_doc[f] = v.get_value(record)
        return new_doc


@dataclass
class EvtxHandler:
    source: Path
    filters: List[Filter]
    filter_op: Any
    transformer: DocumentTransformer

    @staticmethod
    def from_source(source: Union[Path, str]) -> 'EvtxHandler':
        """Create a default EvtxHandler from a given source.
        """
        source = Path(source)
        filters = []
        filter_op = any
        transformer = DocumentTransformer.empty()

        return EvtxHandler(source, filters, filter_op, transformer)
    
    def with_transformer(self, transformer: DocumentTransformer) -> 'EvtxHandler':
        """Add a transformer to the EvtxHandler.
        """
        self.transformer = transformer
        return self

    def with_filter(self, filter_: Filter) -> 'EvtxHandler':
        """Add a filter to the EvtxHandler.
        """
        self.filters.append(filter_)
        return self
    
    def parse_into_dataframe(self) -> pd.DataFrame:
        """Parse EVTX records into a DataFrame based off the DocumentTransformer
        and Filters.
        """
        records = []
        for r in _iterate_evtx_records(self.source):
            if self.filters:
                flag_pass = self.filter_op([f.matches(r) for f in self.filters])
            else:
                flag_pass = True

            if flag_pass:
                if self.transformer.mapping:
                    r = self.transformer.get_record(r)

                records.append(r)
        
        df = pd.DataFrame(records)
        return df


def _iterate_evtx_paths(location: Path) -> Iterator[Path]:
    """Iterate over EVTX files in a location, or if the location is a file
    then yield itself.
    """
    if location.is_dir():
        for p in location.glob("**/*.evtx"):
            if p.is_file() and p.stat().st_size > 0:
                yield p
    else:
        yield location


def _iterate_evtx_records(source: Path):
    """Given a location, iterate over evtx records.
    """
    for evtx_file in _iterate_evtx_paths(source):
        with evtx_file.open("rb") as fh:
            evtx_parser = PyEvtxParser(fh)
            for record in evtx_parser.records_json():
                data = ujson.loads(record["data"])
                data["_source"] = evtx_file.name
                yield data
