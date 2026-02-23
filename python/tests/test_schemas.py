"""Pydantic schema validation tests including hypothesis property-based tests."""

from __future__ import annotations

import pytest
from hypothesis import given, settings
from hypothesis import strategies as st
from pydantic import ValidationError

from server.schemas import (
    SUPPORTED_LANGUAGES,
    CloneRequest,
    CustomVoiceRequest,
    MultiSpeakerRequest,
    MultiSpeakerSegment,
    RenameRequest,
    VoiceDesignRequest,
)

# ─── Strategy helpers ───────────────────────────────────────────

valid_language = st.sampled_from(SUPPORTED_LANGUAGES)
valid_text = st.text(min_size=1, max_size=100).filter(lambda s: len(s.strip()) > 0)
valid_instruct = st.text(min_size=1, max_size=100).filter(lambda s: len(s.strip()) > 0)
valid_ref_id = st.text(min_size=1, max_size=50).filter(lambda s: len(s.strip()) > 0)
valid_speaker = st.sampled_from(["Chelsie", "Aidan", "Aaliyah", "Ethan"])


# ─── CloneRequest ───────────────────────────────────────────────


class TestCloneRequest:
    def test_valid_minimal(self) -> None:
        req = CloneRequest(text="Hello", ref_audio_id="abc123")
        assert req.language == "auto"
        assert req.ref_text is None

    def test_valid_full(self) -> None:
        req = CloneRequest(
            text="Hello",
            ref_audio_id="abc123",
            ref_text="reference",
            language="English",
        )
        assert req.ref_text == "reference"
        assert req.language == "English"

    def test_empty_text_rejected(self) -> None:
        with pytest.raises(ValidationError):
            CloneRequest(text="", ref_audio_id="abc123")

    def test_empty_ref_audio_id_rejected(self) -> None:
        with pytest.raises(ValidationError):
            CloneRequest(text="Hello", ref_audio_id="")

    def test_invalid_language_rejected(self) -> None:
        with pytest.raises(ValidationError):
            CloneRequest(text="Hello", ref_audio_id="abc", language="Klingon")

    def test_text_too_long_rejected(self) -> None:
        with pytest.raises(ValidationError):
            CloneRequest(text="x" * 10001, ref_audio_id="abc")

    @given(text=valid_text, ref_id=valid_ref_id, lang=valid_language)
    @settings(max_examples=50)
    def test_valid_inputs_always_parse(self, text: str, ref_id: str, lang: str) -> None:
        req = CloneRequest(text=text, ref_audio_id=ref_id, language=lang)
        assert len(req.text) >= 1
        assert req.language in SUPPORTED_LANGUAGES

    @given(lang=st.text(min_size=1, max_size=50))
    @settings(max_examples=50)
    def test_random_language_rejected_unless_supported(self, lang: str) -> None:
        if lang in SUPPORTED_LANGUAGES:
            req = CloneRequest(text="Hello", ref_audio_id="abc", language=lang)
            assert req.language == lang
        else:
            with pytest.raises(ValidationError):
                CloneRequest(text="Hello", ref_audio_id="abc", language=lang)


# ─── VoiceDesignRequest ─────────────────────────────────────────


class TestVoiceDesignRequest:
    def test_valid(self) -> None:
        req = VoiceDesignRequest(text="Hello", instruct="warm voice")
        assert req.language == "auto"

    def test_empty_text_rejected(self) -> None:
        with pytest.raises(ValidationError):
            VoiceDesignRequest(text="", instruct="warm")

    def test_empty_instruct_rejected(self) -> None:
        with pytest.raises(ValidationError):
            VoiceDesignRequest(text="Hello", instruct="")

    def test_instruct_too_long_rejected(self) -> None:
        with pytest.raises(ValidationError):
            VoiceDesignRequest(text="Hello", instruct="x" * 1001)

    @given(text=valid_text, instruct=valid_instruct, lang=valid_language)
    @settings(max_examples=50)
    def test_valid_inputs_always_parse(
        self, text: str, instruct: str, lang: str
    ) -> None:
        req = VoiceDesignRequest(text=text, instruct=instruct, language=lang)
        assert len(req.text) >= 1
        assert len(req.instruct) >= 1


# ─── CustomVoiceRequest ─────────────────────────────────────────


class TestCustomVoiceRequest:
    def test_valid_minimal(self) -> None:
        req = CustomVoiceRequest(text="Hello", speaker="Chelsie")
        assert req.language == "auto"
        assert req.instruct is None

    def test_valid_with_instruct(self) -> None:
        req = CustomVoiceRequest(
            text="Hello", speaker="Ethan", instruct="slowly"
        )
        assert req.instruct == "slowly"

    def test_empty_text_rejected(self) -> None:
        with pytest.raises(ValidationError):
            CustomVoiceRequest(text="", speaker="Chelsie")

    def test_empty_speaker_rejected(self) -> None:
        with pytest.raises(ValidationError):
            CustomVoiceRequest(text="Hello", speaker="")

    @given(text=valid_text, speaker=valid_speaker, lang=valid_language)
    @settings(max_examples=50)
    def test_valid_inputs_always_parse(
        self, text: str, speaker: str, lang: str
    ) -> None:
        req = CustomVoiceRequest(text=text, speaker=speaker, language=lang)
        assert len(req.text) >= 1


# ─── MultiSpeakerRequest ────────────────────────────────────────


class TestMultiSpeakerRequest:
    def test_valid_single_segment(self) -> None:
        req = MultiSpeakerRequest(
            segments=[
                MultiSpeakerSegment(text="Hello", ref_audio_id="abc")
            ]
        )
        assert len(req.segments) == 1

    def test_empty_segments_rejected(self) -> None:
        with pytest.raises(ValidationError):
            MultiSpeakerRequest(segments=[])

    def test_too_many_segments_rejected(self) -> None:
        segments = [
            MultiSpeakerSegment(text="Hello", ref_audio_id=f"id{i}")
            for i in range(101)
        ]
        with pytest.raises(ValidationError):
            MultiSpeakerRequest(segments=segments)

    @given(
        count=st.integers(min_value=1, max_value=5),
        text=valid_text,
        ref_id=valid_ref_id,
    )
    @settings(max_examples=20)
    def test_variable_segment_count(
        self, count: int, text: str, ref_id: str
    ) -> None:
        segments = [
            MultiSpeakerSegment(text=text, ref_audio_id=ref_id)
            for _ in range(count)
        ]
        req = MultiSpeakerRequest(segments=segments)
        assert len(req.segments) == count


# ─── RenameRequest ──────────────────────────────────────────────


class TestRenameRequest:
    def test_valid(self) -> None:
        req = RenameRequest(name="My Voice")
        assert req.name == "My Voice"

    def test_empty_name_rejected(self) -> None:
        with pytest.raises(ValidationError):
            RenameRequest(name="")

    def test_name_too_long_rejected(self) -> None:
        with pytest.raises(ValidationError):
            RenameRequest(name="x" * 201)

    @given(name=st.text(min_size=1, max_size=200).filter(lambda s: len(s.strip()) > 0))
    @settings(max_examples=50)
    def test_valid_names_always_parse(self, name: str) -> None:
        req = RenameRequest(name=name)
        assert len(req.name) >= 1
