# Copyright 2024 Felix Engl
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

from .ldatranslate import PyTopicModel, PyVocabulary, LanguageHint
from typing import Protocol, Iterable, Any

class TomotopyDocAlike(Protocol):
    def __len__(self) -> int:...
    def get_topic_dist(self) -> Iterable[Any | float]:...


class TomotopyModelAlike(Protocol):
    used_vocabs: Iterable[Any | str]
    k: int
    docs: Iterable[TomotopyDocAlike]
    used_vocab_freq: Iterable[Any |int]
    def get_topic_word_dist(self, topic: int) -> Iterable[Any | float]: ...


def tomotopy_to_topic_model(model: TomotopyModelAlike, language: None | str | LanguageHint = None) -> PyTopicModel:
    """
    Allows to convert a tomotopy alike model to a PyTopicModel
    """
    vocabulary = PyVocabulary(language, [str(x) for x in model.used_vocabs])
    topics = [[float(value) for value in model.get_topic_word_dist(k)] for k in range(0, model.k)]
    doc_lengths = [len(doc) for doc in model.docs]
    doc_topic_dists = [[float(x) for x in doc.get_topic_dist()] for doc in model.docs]
    term_frequency = [int(x) for x in model.used_vocab_freq]
    for topic in topics:
        assert len(topic) == len(vocabulary)
    return PyTopicModel(topics, vocabulary, term_frequency, doc_topic_dists, doc_lengths)
