using System;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200009D RID: 157
	[HandlerCategory("vvPosClose"), HandlerName("Трейл по теням True")]
	public class TrailOnShadowsT : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs, IContextUses
	{
		// Token: 0x060005B0 RID: 1456 RVA: 0x0001C054 File Offset: 0x0001A254
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFEPct(barNum);
			ISecurity security = pos.get_Security();
			double num3;
			if (num > this.TrailEnable)
			{
				int num2 = Math.Min(this.Bars_TrailStop, barNum);
				num3 = (pos.get_IsLong() ? Indicators.Lowest(security.get_LowPrices(), barNum, num2) : Indicators.Highest(security.get_HighPrices(), barNum, num2));
			}
			else
			{
				int num4 = Math.Min(this.Bars_StopLoss, pos.get_EntryBarNum());
				num3 = (pos.get_IsLong() ? Indicators.Lowest(security.get_LowPrices(), pos.get_EntryBarNum(), num4) : Indicators.Highest(security.get_HighPrices(), pos.get_EntryBarNum(), num4));
			}
			double stop = pos.GetStop(barNum);
			if (stop == 0.0)
			{
				return num3;
			}
			if (num <= this.TrailEnable)
			{
				return num3;
			}
			if (!pos.get_IsLong())
			{
				return Math.Min(num3, stop);
			}
			return Math.Max(num3, stop);
		}

		// Token: 0x170001ED RID: 493
		[HandlerParameter(true, "5", Min = "1", Max = "20", Step = "1", Name = "Баров - СтопЛосс")]
		public int Bars_StopLoss
		{
			// Token: 0x060005AA RID: 1450 RVA: 0x0001C01E File Offset: 0x0001A21E
			get;
			// Token: 0x060005AB RID: 1451 RVA: 0x0001C026 File Offset: 0x0001A226
			set;
		}

		// Token: 0x170001EF RID: 495
		[HandlerParameter(true, "15", Min = "1", Max = "20", Step = "1", Name = "Баров - ТрейлСтоп")]
		public int Bars_TrailStop
		{
			// Token: 0x060005AE RID: 1454 RVA: 0x0001C040 File Offset: 0x0001A240
			get;
			// Token: 0x060005AF RID: 1455 RVA: 0x0001C048 File Offset: 0x0001A248
			set;
		}

		// Token: 0x170001F0 RID: 496
		public IContext Context
		{
			// Token: 0x060005B1 RID: 1457 RVA: 0x0001C144 File Offset: 0x0001A344
			get;
			// Token: 0x060005B2 RID: 1458 RVA: 0x0001C14C File Offset: 0x0001A34C
			set;
		}

		// Token: 0x170001EE RID: 494
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.7", Step = "0.1", Name = "Trail Enable")]
		public double TrailEnable
		{
			// Token: 0x060005AC RID: 1452 RVA: 0x0001C02F File Offset: 0x0001A22F
			get;
			// Token: 0x060005AD RID: 1453 RVA: 0x0001C037 File Offset: 0x0001A237
			set;
		}
	}
}
