using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200004F RID: 79
	[HandlerCategory("vvIndicators"), HandlerName("RS Rank")]
	public class RSRank : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060002D2 RID: 722 RVA: 0x0000D840 File Offset: 0x0000BA40
		public IList<double> Execute(ISecurity src)
		{
			int count = src.get_Bars().Count;
			int num = Math.Max(this.ShortTermPeriod, this.LongTermPeriod);
			double[] array = new double[count];
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> data = this.Context.GetData("atr", new string[]
			{
				this.AtrPeriod.ToString(),
				src.get_CacheName()
			}, () => ATR.ATR_TSLab(src.get_Bars(), this.AtrPeriod));
			for (int i = num; i < count; i++)
			{
				double num2 = (closePrices[i] - closePrices[i - this.LongTermPeriod] + (closePrices[i] - closePrices[i - this.ShortTermPeriod])) / 2.0;
				if (data[i] > 0.0)
				{
					num2 /= data[i];
				}
				else
				{
					num2 = 0.0;
				}
				array[i] = num2;
			}
			for (int j = 0; j < num; j++)
			{
				array[j] = array[num + 1];
			}
			if (this.postSmooth > 0)
			{
				return JMA.GenJMA(array, this.postSmooth, 100);
			}
			return array;
		}

		// Token: 0x170000F3 RID: 243
		[HandlerParameter(true, "10", Min = "1", Max = "100", Step = "1")]
		public int AtrPeriod
		{
			// Token: 0x060002CE RID: 718 RVA: 0x0000D7F7 File Offset: 0x0000B9F7
			get;
			// Token: 0x060002CF RID: 719 RVA: 0x0000D7FF File Offset: 0x0000B9FF
			set;
		}

		// Token: 0x170000F5 RID: 245
		public IContext Context
		{
			// Token: 0x060002D3 RID: 723 RVA: 0x0000D9A7 File Offset: 0x0000BBA7
			get;
			// Token: 0x060002D4 RID: 724 RVA: 0x0000D9AF File Offset: 0x0000BBAF
			set;
		}

		// Token: 0x170000F2 RID: 242
		[HandlerParameter(true, "100", Min = "1", Max = "100", Step = "1")]
		public int LongTermPeriod
		{
			// Token: 0x060002CC RID: 716 RVA: 0x0000D7E6 File Offset: 0x0000B9E6
			get;
			// Token: 0x060002CD RID: 717 RVA: 0x0000D7EE File Offset: 0x0000B9EE
			set;
		}

		// Token: 0x170000F4 RID: 244
		[HandlerParameter(true, "0", Min = "0", Max = "10", Step = "1")]
		public int postSmooth
		{
			// Token: 0x060002D0 RID: 720 RVA: 0x0000D808 File Offset: 0x0000BA08
			get;
			// Token: 0x060002D1 RID: 721 RVA: 0x0000D810 File Offset: 0x0000BA10
			set;
		}

		// Token: 0x170000F1 RID: 241
		[HandlerParameter(true, "15", Min = "1", Max = "30", Step = "1")]
		public int ShortTermPeriod
		{
			// Token: 0x060002CA RID: 714 RVA: 0x0000D7D5 File Offset: 0x0000B9D5
			get;
			// Token: 0x060002CB RID: 715 RVA: 0x0000D7DD File Offset: 0x0000B9DD
			set;
		}
	}
}
