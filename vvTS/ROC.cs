using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200004E RID: 78
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("RoC")]
	public class ROC : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060002C6 RID: 710 RVA: 0x0000D72C File Offset: 0x0000B92C
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("RoC", new string[]
			{
				this.Period.ToString(),
				this.Smooth.ToString(),
				this.Abs.ToString(),
				src.GetHashCode().ToString()
			}, () => ROC.GenROC(src, this.Period, this.Smooth, this.Abs));
		}

		// Token: 0x060002C5 RID: 709 RVA: 0x0000D674 File Offset: 0x0000B874
		public static IList<double> GenROC(IList<double> src, int period = 15, int smooth = 0, bool _Abs = false)
		{
			double[] array = new double[src.Count];
			for (int i = period; i < src.Count; i++)
			{
				if (i <= period)
				{
					array[i] = 0.0;
				}
				else
				{
					array[i] = (src[i] - src[i - period]) / src[i - period] * 100.0;
				}
				if (_Abs)
				{
					array[i] = Math.Abs(array[i]);
				}
			}
			IList<double> result = array;
			if (smooth > 0)
			{
				result = JMA.GenJMA(array, smooth, 0);
			}
			return result;
		}

		// Token: 0x170000EF RID: 239
		[HandlerParameter(true, "false", NotOptimized = true)]
		public bool Abs
		{
			// Token: 0x060002C3 RID: 707 RVA: 0x0000D661 File Offset: 0x0000B861
			get;
			// Token: 0x060002C4 RID: 708 RVA: 0x0000D669 File Offset: 0x0000B869
			set;
		}

		// Token: 0x170000F0 RID: 240
		public IContext Context
		{
			// Token: 0x060002C7 RID: 711 RVA: 0x0000D7BC File Offset: 0x0000B9BC
			get;
			// Token: 0x060002C8 RID: 712 RVA: 0x0000D7C4 File Offset: 0x0000B9C4
			set;
		}

		// Token: 0x170000ED RID: 237
		[HandlerParameter(true, "15", Min = "1", Max = "100", Step = "1")]
		public int Period
		{
			// Token: 0x060002BF RID: 703 RVA: 0x0000D63F File Offset: 0x0000B83F
			get;
			// Token: 0x060002C0 RID: 704 RVA: 0x0000D647 File Offset: 0x0000B847
			set;
		}

		// Token: 0x170000EE RID: 238
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x060002C1 RID: 705 RVA: 0x0000D650 File Offset: 0x0000B850
			get;
			// Token: 0x060002C2 RID: 706 RVA: 0x0000D658 File Offset: 0x0000B858
			set;
		}
	}
}
