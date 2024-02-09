using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000113 RID: 275
	[HandlerCategory("vvTrade"), InputsCount(2, 6)]
	public class Max_v : IDoubleAccumHandler, IStreamHandler, IDouble2CalculatorHandler, ITwoSourcesHandler, IDoubleReturns, IValuesHandler, IHandler, IDoubleInputs
	{
		// Token: 0x060007AC RID: 1964 RVA: 0x00021A8E File Offset: 0x0001FC8E
		public double Execute(double source1, double source2)
		{
			return Math.Max(source1, source2);
		}

		// Token: 0x060007B1 RID: 1969 RVA: 0x00021AE3 File Offset: 0x0001FCE3
		public IList<double> Execute(IList<double> source1, IList<double> source2)
		{
			return Series.Max(source1, source2);
		}

		// Token: 0x060007AD RID: 1965 RVA: 0x00021A97 File Offset: 0x0001FC97
		public double Execute(double source1, double source2, double source3)
		{
			return this.Execute(Math.Max(source1, source2), source3);
		}

		// Token: 0x060007B2 RID: 1970 RVA: 0x00021AEC File Offset: 0x0001FCEC
		public IList<double> Execute(IList<double> source1, IList<double> source2, IList<double> source3)
		{
			return Series.Max(this.Execute(source3, source2), source1);
		}

		// Token: 0x060007AE RID: 1966 RVA: 0x00021AA7 File Offset: 0x0001FCA7
		public double Execute(double source1, double source2, double source3, double source4)
		{
			return this.Execute(Math.Max(source1, source2), source3, source4);
		}

		// Token: 0x060007B3 RID: 1971 RVA: 0x00021AFC File Offset: 0x0001FCFC
		public IList<double> Execute(IList<double> source1, IList<double> source2, IList<double> source3, IList<double> source4)
		{
			return Series.Max(this.Execute(source4, source3, source2), source1);
		}

		// Token: 0x060007AF RID: 1967 RVA: 0x00021AB9 File Offset: 0x0001FCB9
		public double Execute(double source1, double source2, double source3, double source4, double source5)
		{
			return this.Execute(Math.Max(source1, source2), source3, source4, source5);
		}

		// Token: 0x060007B4 RID: 1972 RVA: 0x00021B0E File Offset: 0x0001FD0E
		public IList<double> Execute(IList<double> source1, IList<double> source2, IList<double> source3, IList<double> source4, IList<double> source5)
		{
			return Series.Max(this.Execute(source5, source4, source3, source2), source1);
		}

		// Token: 0x060007B0 RID: 1968 RVA: 0x00021ACD File Offset: 0x0001FCCD
		public double Execute(double source1, double source2, double source3, double source4, double source5, double source6)
		{
			return this.Execute(Math.Max(source1, source2), source3, source4, source5, source6);
		}

		// Token: 0x060007B5 RID: 1973 RVA: 0x00021B22 File Offset: 0x0001FD22
		public IList<double> Execute(IList<double> source1, IList<double> source2, IList<double> source3, IList<double> source4, IList<double> source5, IList<double> source6)
		{
			return Series.Max(this.Execute(source6, source5, source4, source3, source2), source1);
		}
	}
}
