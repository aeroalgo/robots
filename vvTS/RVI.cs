using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000053 RID: 83
	[HandlerCategory("vvIndicators"), HandlerName("RVI (Relative Vigor Index)")]
	public class RVI : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060002F4 RID: 756 RVA: 0x0000E2D3 File Offset: 0x0000C4D3
		public IList<double> Execute(ISecurity src)
		{
			return RVI.GenRVI(src, this.Length, this.Trigger);
		}

		// Token: 0x060002F3 RID: 755 RVA: 0x0000E0F4 File Offset: 0x0000C2F4
		public static IList<double> GenRVI(ISecurity src, int length, bool trigger)
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
			for (int i = length + 1; i < count; i++)
			{
				array3[i] = (closePrices[i] - openPrices[i] + 2.0 * (closePrices[i - 1] - openPrices[i - 1]) + 2.0 * (closePrices[i - 2] - openPrices[i - 2]) + (closePrices[i - 3] - openPrices[i - 3])) / 6.0;
				array4[i] = (highPrices[i] - lowPrices[i] + 2.0 * (highPrices[i - 1] - lowPrices[i - 1]) + 2.0 * (highPrices[i - 2] - lowPrices[i - 2]) + (highPrices[i - 3] - lowPrices[i - 3])) / 6.0;
				double num = 0.0;
				double num2 = 0.0;
				for (int j = 0; j < length; j++)
				{
					num += array3[i - j];
					num2 += array4[i - j];
				}
				if (num2 != 0.0)
				{
					array[i] = num / num2;
				}
				else
				{
					array[i] = 0.0;
				}
				array2[i] = array[i - 1];
			}
			if (!trigger)
			{
				return array;
			}
			return array2;
		}

		// Token: 0x17000100 RID: 256
		public IContext Context
		{
			// Token: 0x060002F5 RID: 757 RVA: 0x0000E2E7 File Offset: 0x0000C4E7
			get;
			// Token: 0x060002F6 RID: 758 RVA: 0x0000E2EF File Offset: 0x0000C4EF
			set;
		}

		// Token: 0x170000FE RID: 254
		[HandlerParameter(true, "20", Min = "1", Max = "50", Step = "1")]
		public int Length
		{
			// Token: 0x060002EF RID: 751 RVA: 0x0000E0D0 File Offset: 0x0000C2D0
			get;
			// Token: 0x060002F0 RID: 752 RVA: 0x0000E0D8 File Offset: 0x0000C2D8
			set;
		}

		// Token: 0x170000FF RID: 255
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Trigger
		{
			// Token: 0x060002F1 RID: 753 RVA: 0x0000E0E1 File Offset: 0x0000C2E1
			get;
			// Token: 0x060002F2 RID: 754 RVA: 0x0000E0E9 File Offset: 0x0000C2E9
			set;
		}
	}
}
