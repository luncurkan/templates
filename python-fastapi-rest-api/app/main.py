import os
from contextlib import asynccontextmanager
from datetime import datetime
from typing import Optional

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
from sqlalchemy import Column, DateTime, Integer, String, Text, select
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import DeclarativeBase

# Environment
DATABASE_URL = os.getenv("DATABASE_URL", "sqlite+aiosqlite:///./data.db")

# Database setup
engine = create_async_engine(DATABASE_URL, echo=False)
async_session = async_sessionmaker(engine, expire_on_commit=False)


class Base(DeclarativeBase):
    pass


class Item(Base):
    __tablename__ = "items"

    id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column(String(255), nullable=False)
    description = Column(Text, nullable=True)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)


# Schemas
class ItemCreate(BaseModel):
    name: str = Field(..., min_length=1, max_length=255)
    description: Optional[str] = Field(None, max_length=1000)


class ItemUpdate(BaseModel):
    name: Optional[str] = Field(None, min_length=1, max_length=255)
    description: Optional[str] = Field(None, max_length=1000)


class ItemResponse(BaseModel):
    id: int
    name: str
    description: Optional[str]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


# Lifespan
@asynccontextmanager
async def lifespan(app: FastAPI):
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
    yield


# App
app = FastAPI(title="REST API", lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health")
async def health():
    return {"status": "ok"}


@app.get("/api/items", response_model=list[ItemResponse])
async def list_items():
    async with async_session() as session:
        result = await session.execute(select(Item).order_by(Item.created_at.desc()))
        return result.scalars().all()


@app.get("/api/items/{item_id}", response_model=ItemResponse)
async def get_item(item_id: int):
    async with async_session() as session:
        item = await session.get(Item, item_id)
        if not item:
            raise HTTPException(status_code=404, detail="Item not found")
        return item


@app.post("/api/items", response_model=ItemResponse, status_code=201)
async def create_item(data: ItemCreate):
    async with async_session() as session:
        item = Item(name=data.name, description=data.description)
        session.add(item)
        await session.commit()
        await session.refresh(item)
        return item


@app.put("/api/items/{item_id}", response_model=ItemResponse)
async def update_item(item_id: int, data: ItemUpdate):
    async with async_session() as session:
        item = await session.get(Item, item_id)
        if not item:
            raise HTTPException(status_code=404, detail="Item not found")

        if data.name is not None:
            item.name = data.name
        if data.description is not None:
            item.description = data.description

        await session.commit()
        await session.refresh(item)
        return item


@app.delete("/api/items/{item_id}")
async def delete_item(item_id: int):
    async with async_session() as session:
        item = await session.get(Item, item_id)
        if not item:
            raise HTTPException(status_code=404, detail="Item not found")

        await session.delete(item)
        await session.commit()
        return {"message": "Item deleted"}
