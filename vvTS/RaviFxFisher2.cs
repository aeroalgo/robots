using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200004C RID: 76
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("Ravi FxFisher2")]
	public class RaviFxFisher2 : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060002B7 RID: 695 RVA: 0x0000D0BC File Offset: 0x0000B2BC
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("RaviFxFisher2", new string[]
			{
				this.FastMAperiod.ToString(),
				this.SlowMAperiod.ToString(),
				sec.get_CacheName()
			}, () => RaviFxFisher2.GenRaviFxFisher2(sec, this.FastMAperiod, this.SlowMAperiod, this.Context));
		}

		// Token: 0x060002B6 RID: 694 RVA: 0x0000CE24 File Offset: 0x0000B024
		public static IList<double> GenRaviFxFisher2(ISecurity sec, int _FastMAperiod, int _SlowMAperiod, IContext ctx)
		{
			IList<double> list = new List<double>(sec.get_Bars().Count);
			IList<double> typprice = ctx.GetData("TypicalPrice", new string[]
			{
				sec.get_CacheName()
			}, () => vvSeries.TypicalPrice(sec.get_Bars()));
			IList<double> data = ctx.GetData("lwma", new string[]
			{
				_FastMAperiod.ToString(),
				typprice.GetHashCode().ToString()
			}, () => LWMA.GenWMA(typprice, _FastMAperiod));
			IList<double> data2 = ctx.GetData("lwma", new string[]
			{
				_SlowMAperiod.ToString(),
				typprice.GetHashCode().ToString()
			}, () => LWMA.GenWMA(typprice, _SlowMAperiod));
			IList<double> data3 = ctx.GetData("atr", new string[]
			{
				_FastMAperiod.ToString(),
				sec.get_CacheName()
			}, () => ATR.GenATR(sec.get_Bars(), _FastMAperiod));
			ctx.GetData("atr", new string[]
			{
				_SlowMAperiod.ToString(),
				sec.get_CacheName()
			}, () => ATR.GenATR(sec.get_Bars(), _SlowMAperiod));
			for (int i = 0; i < _SlowMAperiod; i++)
			{
				list.Add(0.0);
			}
			for (int j = _SlowMAperiod; j < sec.get_Bars().Count; j++)
			{
				double num = 100.0 * (data[j] - data2[j]) * data3[j] / data2[j] / data3[j];
				double item = (Math.Exp(2.0 * num) - 1.0) / (Math.Exp(2.0 * num) + 1.0);
				list.Add(item);
			}
			return list;
		}

		// Token: 0x170000EB RID: 235
		public IContext Context
		{
			// Token: 0x060002B8 RID: 696 RVA: 0x0000D131 File Offset: 0x0000B331
			get;
			// Token: 0x060002B9 RID: 697 RVA: 0x0000D139 File Offset: 0x0000B339
			set;
		}

		// Token: 0x170000E9 RID: 233
		[HandlerParameter(true, "4", Min = "1", Max = "50", Step = "1")]
		public int FastMAperiod
		{
			// Token: 0x060002B2 RID: 690 RVA: 0x0000CD92 File Offset: 0x0000AF92
			get;
			// Token: 0x060002B3 RID: 691 RVA: 0x0000CD9A File Offset: 0x0000AF9A
			set;
		}

		// Token: 0x170000EA RID: 234
		[HandlerParameter(true, "49", Min = "1", Max = "120", Step = "1")]
		public int SlowMAperiod
		{
			// Token: 0x060002B4 RID: 692 RVA: 0x0000CDA3 File Offset: 0x0000AFA3
			get;
			// Token: 0x060002B5 RID: 693 RVA: 0x0000CDAB File Offset: 0x0000AFAB
			set;
		}
	}
}
