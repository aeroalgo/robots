using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000054 RID: 84
	[HandlerCategory("vvIndicators"), HandlerName("Fisher RVI")]
	public class FisherRVI : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060002FD RID: 765 RVA: 0x0000E625 File Offset: 0x0000C825
		public IList<double> Execute(ISecurity src)
		{
			return this.GenFisherRVI(src, this.Length, this.Trigger);
		}

		// Token: 0x060002FC RID: 764 RVA: 0x0000E324 File Offset: 0x0000C524
		public IList<double> GenFisherRVI(ISecurity src, int length, bool trigger)
		{
			if (length < 4)
			{
				return null;
			}
			int count = src.get_Bars().Count;
			IList<double> openPrices = src.get_OpenPrices();
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> highPrices = src.get_HighPrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			double[] array7 = new double[count];
			for (int i = length + 1; i < count; i++)
			{
				array4[i] = (closePrices[i] - openPrices[i] + 2.0 * (closePrices[i - 1] - openPrices[i - 1]) + 2.0 * (closePrices[i - 2] - openPrices[i - 2]) + (closePrices[i - 3] - openPrices[i - 3])) / 6.0;
				array5[i] = (highPrices[i] - lowPrices[i] + 2.0 * (highPrices[i - 1] - lowPrices[i - 1]) + 2.0 * (highPrices[i - 2] - lowPrices[i - 2]) + (highPrices[i - 3] - lowPrices[i - 3])) / 6.0;
				double num = 0.0;
				double num2 = 0.0;
				for (int j = 0; j < length; j++)
				{
					num += array4[i - j];
					num2 += array5[i - j];
				}
				if (num2 != 0.0)
				{
					array2[i] = num / num2;
				}
				else
				{
					array2[i] = 0.0;
				}
				double num3 = array2[i];
				double num4 = array2[i];
				for (int k = 0; k < length; k++)
				{
					double val = array2[i - k];
					num3 = Math.Max(num3, val);
					num4 = Math.Min(num4, val);
				}
				array6[i] = 0.0;
				if (num3 != num4)
				{
					array6[i] = (array2[i] - num4) / (num3 - num4);
				}
				array7[i] = (4.0 * array6[i] + 3.0 * array6[i - 1] + 2.0 * array6[i - 2] + array6[i - 3]) / 10.0;
				array[i] = 0.5 * Math.Log((1.0 + 1.98 * (array7[i] - 0.5)) / (1.0 - 1.98 * (array7[i] - 0.5)));
				array3[i] = array[i - 1];
			}
			if (!trigger)
			{
				return array;
			}
			return array3;
		}

		// Token: 0x17000103 RID: 259
		public IContext Context
		{
			// Token: 0x060002FE RID: 766 RVA: 0x0000E63A File Offset: 0x0000C83A
			get;
			// Token: 0x060002FF RID: 767 RVA: 0x0000E642 File Offset: 0x0000C842
			set;
		}

		// Token: 0x17000101 RID: 257
		[HandlerParameter(true, "8", Min = "1", Max = "50", Step = "1")]
		public int Length
		{
			// Token: 0x060002F8 RID: 760 RVA: 0x0000E300 File Offset: 0x0000C500
			get;
			// Token: 0x060002F9 RID: 761 RVA: 0x0000E308 File Offset: 0x0000C508
			set;
		}

		// Token: 0x17000102 RID: 258
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Trigger
		{
			// Token: 0x060002FA RID: 762 RVA: 0x0000E311 File Offset: 0x0000C511
			get;
			// Token: 0x060002FB RID: 763 RVA: 0x0000E319 File Offset: 0x0000C519
			set;
		}
	}
}
