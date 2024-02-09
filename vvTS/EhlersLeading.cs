using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000180 RID: 384
	[HandlerCategory("vvAverages"), HandlerName("Leading Indicator")]
	public class EhlersLeading : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000C27 RID: 3111 RVA: 0x00034C58 File Offset: 0x00032E58
		public IList<double> Execute(ISecurity src)
		{
			return this.Context.GetData("eli", new string[]
			{
				this.Alpha1.ToString(),
				this.Alpha2.ToString(),
				this.SignalLine.ToString(),
				src.GetHashCode().ToString()
			}, () => EhlersLeading.GenELI(src, 0, this.Alpha1, this.Alpha2, this.SignalLine));
		}

		// Token: 0x06000C26 RID: 3110 RVA: 0x00034B00 File Offset: 0x00032D00
		public static IList<double> GenELI(ISecurity src, int period, double _Alpha1, double _Alpha2, bool _SignalLine)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			IList<double> highPrices = src.get_HighPrices();
			IList<double> lowPrices = src.get_LowPrices();
			for (int i = 1; i < count; i++)
			{
				array[i] = 2.0 * ((highPrices[i] + lowPrices[i]) / 2.0) + (_Alpha1 - 2.0) * ((highPrices[i - 1] + lowPrices[i - 1]) / 2.0) + (1.0 - _Alpha1) * array[i - 1];
				array2[i] = _Alpha2 * array[i] + (1.0 - _Alpha2) * array2[i - 1];
				array3[i] = 0.5 * ((highPrices[i] + lowPrices[i]) / 2.0) + 0.5 * array3[i - 1];
			}
			if (!_SignalLine)
			{
				return array2;
			}
			return array3;
		}

		// Token: 0x170003FA RID: 1018
		[HandlerParameter(true, "0.25", Min = "0.01", Max = "1", Step = "0.01")]
		public double Alpha1
		{
			// Token: 0x06000C20 RID: 3104 RVA: 0x00034ACB File Offset: 0x00032CCB
			get;
			// Token: 0x06000C21 RID: 3105 RVA: 0x00034AD3 File Offset: 0x00032CD3
			set;
		}

		// Token: 0x170003FB RID: 1019
		[HandlerParameter(true, "0.33", Min = "0.01", Max = "1", Step = "0.01")]
		public double Alpha2
		{
			// Token: 0x06000C22 RID: 3106 RVA: 0x00034ADC File Offset: 0x00032CDC
			get;
			// Token: 0x06000C23 RID: 3107 RVA: 0x00034AE4 File Offset: 0x00032CE4
			set;
		}

		// Token: 0x170003FD RID: 1021
		public IContext Context
		{
			// Token: 0x06000C28 RID: 3112 RVA: 0x00034CE8 File Offset: 0x00032EE8
			get;
			// Token: 0x06000C29 RID: 3113 RVA: 0x00034CF0 File Offset: 0x00032EF0
			set;
		}

		// Token: 0x170003FC RID: 1020
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool SignalLine
		{
			// Token: 0x06000C24 RID: 3108 RVA: 0x00034AED File Offset: 0x00032CED
			get;
			// Token: 0x06000C25 RID: 3109 RVA: 0x00034AF5 File Offset: 0x00032CF5
			set;
		}
	}
}
